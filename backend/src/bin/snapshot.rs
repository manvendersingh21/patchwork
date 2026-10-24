#![recursion_limit = "512"]
use anyhow::{bail, Context, Result};
use falkordb::{AsyncGraph, FalkorValue};
use futures_util::StreamExt;
use patchwork::graph::connect;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnapshotNode {
    pub uid: String,
    pub labels: Vec<String>,
    pub properties: BTreeMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnapshotEdge {
    #[serde(rename = "type")]
    pub r#type: String,
    pub src_uid: String,
    pub dst_uid: String,
    pub properties: BTreeMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Snapshot {
    pub nodes: Vec<SnapshotNode>,
    pub edges: Vec<SnapshotEdge>,
}

fn falkor_to_json(v: FalkorValue) -> serde_json::Value {
    match v {
        FalkorValue::String(s) => serde_json::Value::String(s),
        FalkorValue::Bool(b) => serde_json::Value::Bool(b),
        FalkorValue::I64(i) => serde_json::Value::Number(i.into()),
        FalkorValue::F64(f) => serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        FalkorValue::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(falkor_to_json).collect())
        }
        FalkorValue::Map(map) => {
            let mut obj = serde_json::Map::new();
            for (k, val) in map {
                obj.insert(k, falkor_to_json(val));
            }
            serde_json::Value::Object(obj)
        }
        FalkorValue::None => serde_json::Value::Null,
        other => serde_json::Value::String(format!("{other:?}")),
    }
}

fn json_to_falkor(v: &serde_json::Value) -> FalkorValue {
    match v {
        serde_json::Value::Null => FalkorValue::None,
        serde_json::Value::Bool(b) => FalkorValue::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                FalkorValue::I64(i)
            } else if let Some(u) = n.as_u64() {
                FalkorValue::I64(u as i64)
            } else if let Some(f) = n.as_f64() {
                FalkorValue::F64(f)
            } else {
                FalkorValue::None
            }
        }
        serde_json::Value::String(s) => FalkorValue::String(s.clone()),
        serde_json::Value::Array(arr) => {
            FalkorValue::Array(arr.iter().map(json_to_falkor).collect())
        }
        serde_json::Value::Object(map) => {
            let mut m = HashMap::new();
            for (k, val) in map {
                m.insert(k.clone(), json_to_falkor(val));
            }
            FalkorValue::Map(m)
        }
    }
}

async fn export_graph(g: &mut AsyncGraph, out_path: &Path) -> Result<Snapshot> {
    let mut res_nodes = g
        .query("MATCH (n) RETURN labels(n), properties(n)")
        .execute()
        .await?;
    let mut nodes = Vec::new();
    let mut exported_uids = HashSet::new();

    while let Some(row) = res_nodes.data.next().await {
        let mut values = row?.into_values();
        if values.len() < 2 {
            continue;
        }
        let props_val = values.pop().unwrap();
        let labels_val = values.pop().unwrap();

        let mut labels: Vec<String> = match labels_val {
            FalkorValue::Array(arr) => arr
                .into_iter()
                .filter_map(|item| match item {
                    FalkorValue::String(s) => Some(s),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        };
        labels.sort();

        if labels.iter().any(|l| l == "AgentEvent") {
            continue;
        }

        let properties: BTreeMap<String, serde_json::Value> = match props_val {
            FalkorValue::Map(map) => map.into_iter().map(|(k, v)| (k, falkor_to_json(v))).collect(),
            _ => BTreeMap::new(),
        };

        let uid = match properties.get("uid") {
            Some(serde_json::Value::String(s)) => s.clone(),
            _ => continue,
        };

        exported_uids.insert(uid.clone());
        nodes.push(SnapshotNode {
            uid,
            labels,
            properties,
        });
    }

    nodes.sort_by(|a, b| a.uid.cmp(&b.uid));

    let mut res_edges = g
        .query("MATCH (a)-[r]->(b) RETURN a.uid, type(r), b.uid, properties(r)")
        .execute()
        .await?;
    let mut edges = Vec::new();

    while let Some(row) = res_edges.data.next().await {
        let mut values = row?.into_values();
        if values.len() < 4 {
            continue;
        }
        let props_val = values.pop().unwrap();
        let dst_val = values.pop().unwrap();
        let type_val = values.pop().unwrap();
        let src_val = values.pop().unwrap();

        let src_uid = match src_val {
            FalkorValue::String(s) => s,
            _ => continue,
        };
        let r#type = match type_val {
            FalkorValue::String(s) => s,
            _ => continue,
        };
        let dst_uid = match dst_val {
            FalkorValue::String(s) => s,
            _ => continue,
        };

        if !exported_uids.contains(&src_uid) || !exported_uids.contains(&dst_uid) {
            continue;
        }

        let properties: BTreeMap<String, serde_json::Value> = match props_val {
            FalkorValue::Map(map) => map.into_iter().map(|(k, v)| (k, falkor_to_json(v))).collect(),
            _ => BTreeMap::new(),
        };

        edges.push(SnapshotEdge {
            r#type,
            src_uid,
            dst_uid,
            properties,
        });
    }

    edges.sort_by(|a, b| {
        (&a.src_uid, &a.r#type, &a.dst_uid).cmp(&(&b.src_uid, &b.r#type, &b.dst_uid))
    });

    let snapshot = Snapshot { nodes, edges };
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json_text = serde_json::to_string_pretty(&snapshot)?;
    std::fs::write(out_path, json_text)?;

    println!(
        "Exported {} nodes, {} edges to {}",
        snapshot.nodes.len(),
        snapshot.edges.len(),
        out_path.display()
    );

    Ok(snapshot)
}

async fn restore_graph(g: &mut AsyncGraph, in_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(in_path)
        .with_context(|| format!("Failed to read {}", in_path.display()))?;
    let snapshot: Snapshot = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse snapshot JSON in {}", in_path.display()))?;

    // Delete the graph
    let _ = g.delete().await;
    let _ = g.query("MATCH (n) DETACH DELETE n").execute().await;

    // Recreate nodes by uid with MERGE
    for node in &snapshot.nodes {
        let labels_str: String = node
            .labels
            .iter()
            .map(|l| format!(":`{}`", l.replace('`', "")))
            .collect();
        let cypher = format!("MERGE (n{} {{uid: $uid}}) SET n += $props", labels_str);
        let props_map: HashMap<String, FalkorValue> = node
            .properties
            .iter()
            .map(|(k, v)| (k.clone(), json_to_falkor(v)))
            .collect();

        g.query(&cypher)
            .with_param("uid", &node.uid)
            .with_param("props", FalkorValue::Map(props_map))
            .execute()
            .await
            .with_context(|| format!("Failed to restore node {}", node.uid))?;
    }

    // Recreate edges by uid with MERGE
    for edge in &snapshot.edges {
        let rel_type = edge.r#type.replace('`', "");
        // Edges with their own uid (e.g. BRIDGE) are keyed by it, so repeated edges between
        // the same two nodes are kept. Other edges are keyed by type and endpoints.
        let edge_uid = edge.properties.get("uid").and_then(|v| v.as_str()).map(str::to_string);
        let cypher = match edge_uid {
            Some(_) => format!(
                "MATCH (a {{uid: $src}}), (b {{uid: $dst}}) MERGE (a)-[r:`{}` {{uid: $edge_uid}}]->(b) SET r += $props",
                rel_type
            ),
            None => format!(
                "MATCH (a {{uid: $src}}), (b {{uid: $dst}}) MERGE (a)-[r:`{}`]->(b) SET r += $props",
                rel_type
            ),
        };
        let props_map: HashMap<String, FalkorValue> = edge
            .properties
            .iter()
            .map(|(k, v)| (k.clone(), json_to_falkor(v)))
            .collect();

        let mut q = g.query(&cypher)
            .with_param("src", &edge.src_uid)
            .with_param("dst", &edge.dst_uid)
            .with_param("props", FalkorValue::Map(props_map));
        if let Some(uid) = &edge_uid {
            q = q.with_param("edge_uid", uid);
        }
        q.execute()
            .await
            .with_context(|| format!("Failed to restore edge {} -> {}", edge.src_uid, edge.dst_uid))?;
    }

    println!(
        "Restored {} nodes, {} edges from {}",
        snapshot.nodes.len(),
        snapshot.edges.len(),
        in_path.display()
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        bail!("Usage: snapshot <export|restore> <path>");
    }

    let mut g = connect("localhost", 6379, "patchwork").await?;
    let mode = &args[1];
    let path = Path::new(&args[2]);

    match mode.as_str() {
        "export" => {
            export_graph(&mut g, path).await?;
        }
        "restore" => {
            restore_graph(&mut g, path).await?;
        }
        other => {
            bail!("Unknown subcommand '{}'. Expected 'export' or 'restore'.", other);
        }
    }

    Ok(())
}
