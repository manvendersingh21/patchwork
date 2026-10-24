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

pub const T2_TALKS_FOR_ISSUE: &str = "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.uid AS talk_uid, k.title AS talk, t.slug AS topic, s.name AS speaker LIMIT 2";
pub const T3_PEOPLE_TO_MEET: &str = "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE NOT (me)-[:LOGGED]->(:Encounter)-[:MET]->(s) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, s.name AS person, s.company AS company, t.name AS topic, k.title AS talk LIMIT 5";
pub const T4_MEMORY_RECALL: &str = "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3";
pub const T5_ISSUES_FOR_ENCOUNTER: &str = "MATCH p = (e:Encounter {uid: $encounter})-[:DISCUSSED]->(t:Topic)<-[:ABOUT]-(i:Issue {gh_state: 'open'})<-[:HAS_ISSUE]-(:Repo)<-[:MAINTAINS|TRACKS]-(:Person {kind: 'self'}) MATCH (e)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.uid AS issue_uid, i.title AS issue, c.uid AS contact_uid, c.name AS contact, t.name AS topic";
pub const T6_TALKS_BY_TOPIC: &str = "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5";
