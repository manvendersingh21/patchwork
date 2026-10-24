//! FalkorDB connection, run_template(), and the result envelope.
use anyhow::Result;
use falkordb::{AsyncGraph, FalkorAsyncClient, FalkorClientBuilder, FalkorConnectionInfo, FalkorValue};
use futures_util::StreamExt;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};

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

fn prop_str(props: &HashMap<String, FalkorValue>, key: &str) -> String {
    match props.get(key) { Some(FalkorValue::String(s)) => s.clone(), _ => String::new() }
}

/// Walk one returned path into the {nodes, edges} shape. Edges map entity ids to uids.
pub fn serialize_path(p: &falkordb::Path) -> PathJson {
    let mut out = PathJson::default();
    let mut id_to_uid: HashMap<i64, String> = HashMap::new();
    for n in &p.nodes {
        let uid = prop_str(&n.properties, "uid");
        id_to_uid.insert(n.entity_id, uid.clone());
        let name = { let v = prop_str(&n.properties, "name"); if v.is_empty() { prop_str(&n.properties, "title") } else { v } };
        out.nodes.push(PathNode {
            uid,
            label: n.labels.first().cloned().unwrap_or_default(),
            name,
            state: prop_str(&n.properties, "state"),
        });
    }
    for e in &p.relationships {
        out.edges.push(PathEdge {
            kind: e.relationship_type.clone(),
            src_uid: id_to_uid.get(&e.src_node_id).cloned().unwrap_or_default(),
            dst_uid: id_to_uid.get(&e.dst_node_id).cloned().unwrap_or_default(),
        });
    }
    out
}

/// Run a Cypher string with params and wrap the result in the envelope.
/// `params` is a flat object; each value is passed as a typed Falkor parameter.
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
            match v {
                FalkorValue::Path(p) => {
                    let pj = serialize_path(p);
                    hops = hops.max(pj.edges.len());
                    for n in pj.nodes { if seen_nodes.insert(n.uid.clone()) { path.nodes.push(n); } }
                    for e in pj.edges {
                        if seen_edges.insert((e.kind.clone(), e.src_uid.clone(), e.dst_uid.clone())) { path.edges.push(e); }
                    }
                }
                other => { r.insert(col.clone(), falkor_to_json(other)); }
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
