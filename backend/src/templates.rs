//! Cypher templates: the agents' only read tools. Verified against FalkorDB before use.
//! Optional params are passed as empty strings: `$day = ''` means no day filter.
//! The path is returned as plain arrays (nodes, edges), not as a Path value, because the
//! falkordb 0.10.5 client fails to decode Path values.
pub const T1_TALKS_FOR_MY_ISSUES: &str = "\
MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})\
-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) \
WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) \
RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, \
[r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, \
size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, \
k.start_time AS start_time, s.name AS speaker \
ORDER BY day, start_time LIMIT 5";
