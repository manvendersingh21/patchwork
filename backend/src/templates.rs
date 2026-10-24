//! Cypher templates: the agents' only read tools. Verified against FalkorDB before use.
pub const T1_TALKS_FOR_MY_ISSUES: &str = "\
MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})\
-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) \
RETURN p, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker \
ORDER BY day, start_time LIMIT 5";
