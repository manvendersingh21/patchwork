//! FalkorDB connection, run_template(), and the result envelope.
use anyhow::Result;
use falkordb::{AsyncGraph, FalkorAsyncClient, FalkorClientBuilder, FalkorConnectionInfo, FalkorValue};
use futures_util::StreamExt;
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct PathNode { pub uid: String, pub label: String, pub name: String, pub state: String }

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct PathEdge { #[serde(rename = "type")] pub kind: String, pub src_uid: String, pub dst_uid: String }

#[derive(Serialize, Default, Debug, Clone)]
pub struct PathJson { pub nodes: Vec<PathNode>, pub edges: Vec<PathEdge> }

/// The envelope every template run returns.
#[derive(Serialize, Debug)]
pub struct Envelope {
    pub template: String,
    pub params: serde_json::Value,
    pub cypher: String,
    pub hops: usize,
    /// FalkorDB's own internal execution time in ms (not wall clock).
    pub run_time_ms: Option<f64>,
    pub rows: Vec<BTreeMap<String, serde_json::Value>>,
    pub path: PathJson,
}

pub async fn connect(host: &str, port: u16, graph_name: &str) -> Result<AsyncGraph> {
    let info: FalkorConnectionInfo = format!("falkor://{host}:{port}").as_str().try_into()?;
    let client: FalkorAsyncClient = FalkorClientBuilder::new_async().with_connection_info(info).build().await?;
    Ok(client.select_graph(graph_name))
}

fn str_of(v: &FalkorValue) -> String {
    match v { FalkorValue::String(s) => s.clone(), _ => String::new() }
}

fn array_of(v: &FalkorValue) -> &[FalkorValue] {
    match v { FalkorValue::Array(a) => a.as_slice(), _ => &[] }
}

/// Parse the `nodes` column: [[uid, label, name, state], ...].
fn parse_nodes(v: &FalkorValue) -> Vec<PathNode> {
    array_of(v).iter().map(|n| {
        let f = array_of(n);
        PathNode {
            uid: f.first().map(str_of).unwrap_or_default(),
            label: f.get(1).map(str_of).unwrap_or_default(),
            name: f.get(2).map(str_of).unwrap_or_default(),
            state: f.get(3).map(str_of).unwrap_or_default(),
        }
    }).collect()
}

/// Parse the `edges` column: [[type, src_uid, dst_uid], ...].
fn parse_edges(v: &FalkorValue) -> Vec<PathEdge> {
    array_of(v).iter().map(|e| {
        let f = array_of(e);
        PathEdge {
            kind: f.first().map(str_of).unwrap_or_default(),
            src_uid: f.get(1).map(str_of).unwrap_or_default(),
            dst_uid: f.get(2).map(str_of).unwrap_or_default(),
        }
    }).collect()
}

/// Run a Cypher string with params and wrap the result in the envelope.
/// `nodes` and `edges` columns are the path; they are merged across rows and deduped by uid.
pub async fn run_template(
    g: &mut AsyncGraph,
    template: &str,
    cypher: &str,
    params: serde_json::Value,
) -> Result<Envelope> {
    let pairs: Vec<(String, String)> = params
        .as_object()
        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string())).collect())
        .unwrap_or_default();
    let mut res = g.query(cypher).with_params(pairs).execute().await?;
    let run_time_ms = res.get_internal_execution_time();

    let header: Vec<String> = res.header.iter().cloned().collect();
    let mut rows = Vec::new();
    let mut path = PathJson::default();
    let mut hops = 0usize;
    let mut seen_nodes = HashSet::new();
    let mut seen_edges = HashSet::new();
    while let Some(row) = res.data.next().await {
        let values = row?.into_values();
        let mut r = BTreeMap::new();
        for (col, v) in header.iter().zip(values.iter()) {
            match col.as_str() {
                "nodes" => {
                    for n in parse_nodes(v) { if seen_nodes.insert(n.uid.clone()) { path.nodes.push(n); } }
                }
                "edges" => {
                    let edges = parse_edges(v);
                    for e in edges.iter().cloned() {
                        if seen_edges.insert((e.kind.clone(), e.src_uid.clone(), e.dst_uid.clone())) { path.edges.push(e); }
                    }
                }
                "hops" => {
                    if let FalkorValue::I64(h) = v { hops = hops.max(*h as usize); }
                    r.insert(col.clone(), falkor_to_json(v));
                }
                _ => { r.insert(col.clone(), falkor_to_json(v)); }
            }
        }
        if !r.is_empty() { rows.push(r); }
    }
    Ok(Envelope { template: template.to_string(), params, cypher: cypher.to_string(), hops, run_time_ms, rows, path })
}

fn falkor_to_json(v: &FalkorValue) -> serde_json::Value {
    match v {
        FalkorValue::String(s) => serde_json::Value::String(s.clone()),
        FalkorValue::I64(i) => serde_json::Value::from(*i),
        FalkorValue::F64(f) => serde_json::Value::from(*f),
        FalkorValue::Bool(b) => serde_json::Value::Bool(*b),
        _ => serde_json::Value::Null,
    }
}

/// Small raw-query helper for agent claims and writes; does not alter template envelopes.
pub async fn query_rows(g: &mut AsyncGraph, cypher: &str, params: serde_json::Value) -> Result<Vec<BTreeMap<String, serde_json::Value>>> {
    let pairs: Vec<(String, String)> = params.as_object().map(|m| m.iter()
        .map(|(k,v)| (k.clone(), v.as_str().unwrap_or_default().to_string())).collect()).unwrap_or_default();
    let mut res = g.query(cypher).with_params(pairs).execute().await?;
    let header: Vec<String> = res.header.iter().cloned().collect();
    let mut rows = Vec::new();
    while let Some(row) = res.data.next().await {
        let values = row?.into_values();
        rows.push(header.iter().zip(values.iter()).map(|(k,v)| (k.clone(), falkor_to_json(v))).collect());
    }
    Ok(rows)
}

pub async fn write_query(g: &mut AsyncGraph, cypher: &str, params: serde_json::Value) -> Result<()> {
    let pairs: Vec<(String, String)> = params.as_object().map(|m| m.iter()
        .map(|(k,v)| (k.clone(), v.as_str().unwrap_or_default().to_string())).collect()).unwrap_or_default();
    g.query(cypher).with_params(pairs).execute().await?;
    Ok(())
}
