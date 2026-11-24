use super::Node;

pub(super) fn rank(kind: &str) -> usize {
    match kind {
        "part" => 1,
        "chapter" => 2,
        "section" => 3,
        "subsection" => 4,
        _ => unreachable!(),
    }
}

pub(super) struct Ancestor<'a> {
    pub kind: &'static str,
    pub id: &'a str,
    pub number: usize,
}

pub(super) fn walk<'a>(
    nodes: &'a [Node],
    article_number: &mut usize,
    visit: &mut impl FnMut(&'a Node, usize, &[Ancestor<'a>]) -> Result<(), String>,
) -> Result<(), String> {
    fn siblings<'a>(
        nodes: &'a [Node],
        article_number: &mut usize,
        ancestors: &mut Vec<Ancestor<'a>>,
        visit: &mut impl FnMut(&'a Node, usize, &[Ancestor<'a>]) -> Result<(), String>,
    ) -> Result<(), String> {
        let mut counts = [0; 4];
        for node in nodes {
            if let Some((kind, id, _)) = node.upper() {
                let index = rank(kind) - 1;
                counts[index] += 1;
                let number = counts[index];
                visit(node, number, ancestors)?;
                ancestors.push(Ancestor { kind, id, number });
                siblings(node.children(), article_number, ancestors, visit)?;
                ancestors.pop();
            } else if matches!(node, Node::Article { .. }) {
                *article_number += 1;
                visit(node, *article_number, ancestors)?;
            } else {
                return Err("Expected an upper level or article".into());
            }
        }
        Ok(())
    }

    siblings(nodes, article_number, &mut Vec::new(), visit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::{ITEM_PROFILE, Presentation, presentation_ids};
    use serde_json::{Value, json};

    fn article(id: &str, entity: &str) -> Value {
        json!({"kind": "article", "id": id, "title": id, "entities": [entity]})
    }

    #[test]
    fn flat_m_and_xl_round_trip_without_new_fields() {
        let m = r#"{"size":"M","nodes":[{"kind":"chapter","id":"c","title":"C","articles":[{"kind":"article","id":"a","title":"A","entities":["e"]}]}]}"#;
        let xl = r#"{"size":"XL","nodes":[{"kind":"module","id":"m1","title":"M1","file":"one.md","articles":[{"kind":"article","id":"a","title":"A","entities":["e"]}],"decisions":[]},{"kind":"module","id":"m2","title":"M2","file":"two.md","articles":[{"kind":"article","id":"b","title":"B","entities":["f"]}],"decisions":[]}]}"#;
        for original in [m, xl] {
            let presentation: Presentation = serde_json::from_str(original).unwrap();
            assert!(presentation_ids(&presentation).is_ok());
            assert_eq!(serde_json::to_string(&presentation).unwrap(), original);
        }
    }

    #[test]
    fn rejects_invalid_shapes_and_placements() {
        let a = article("a", "e");
        let section = json!({"kind":"section", "id":"s", "title":"S", "children":[a]});
        let chapter = |body: Value| {
            let mut node = json!({"kind":"chapter", "id":"c", "title":"C"});
            node.as_object_mut()
                .unwrap()
                .extend(body.as_object().unwrap().clone());
            node
        };
        let parse = |size: &str, node: Value| {
            serde_json::from_value::<Presentation>(json!({"size":size,"nodes":[node]}))
        };
        assert!(parse("L", chapter(json!({"children":null}))).is_err());
        for node in [
            chapter(json!({})),
            chapter(json!({"articles":[]})),
            chapter(json!({"children":[]})),
            chapter(json!({"articles":[a], "children":[a]})),
            chapter(json!({"articles":[section]})),
            json!({"kind":"section", "id":"s", "title":"S", "children":[chapter(json!({"articles":[a]}))]}),
            json!({"kind":"section", "id":"s", "title":"S", "children":[section]}),
            json!({"kind":"part", "id":"p", "title":"P", "children":[]}),
            json!({"kind":"part", "id":"p", "title":"P", "children":[a, a]}),
        ] {
            assert!(presentation_ids(&parse("L", node).unwrap()).is_err());
        }
        assert!(
            presentation_ids(
                &parse(
                    "M",
                    json!({"kind":"part","id":"p","title":"P","children":[a]})
                )
                .unwrap()
            )
            .is_err()
        );
        assert!(presentation_ids(&parse("S", chapter(json!({"articles":[a]}))).unwrap()).is_err());
    }

    #[test]
    fn walks_in_order_with_local_upper_and_global_article_numbers() {
        let nested = json!({"size":"L", "profile":ITEM_PROFILE, "nodes":[
            {"kind":"part","id":"p","title":"P","children":[
                {"kind":"section","id":"s1","title":"S1","children":[article("a", "e")]},
                {"kind":"chapter","id":"c","title":"C","children":[
                    {"kind":"subsection","id":"sub","title":"Sub","children":[
                        {"kind":"article","id":"b","title":"B","paragraphs":[{"id":"para","items":[{"id":"item","entities":["f"]}]}]}
                    ]}
                ]},
                {"kind":"section","id":"s2","title":"S2","children":[article("d", "g")]}
            ]}
        ]});
        let presentation: Presentation = serde_json::from_value(nested).unwrap();
        assert_eq!(presentation_ids(&presentation).unwrap(), ["e", "f", "g"]);
        assert!(presentation.has_items());
        let mut events = Vec::new();
        let mut number = 0;
        walk(
            &presentation.nodes,
            &mut number,
            &mut |node, n, ancestors| {
                let id = match node {
                    Node::Article { id, .. } => id.as_str(),
                    _ => node.upper().unwrap().1,
                };
                events.push((
                    id.to_string(),
                    n,
                    ancestors
                        .iter()
                        .map(|a| (a.kind, a.id.to_string(), a.number))
                        .collect::<Vec<_>>(),
                ));
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(
            events
                .iter()
                .map(|(id, n, _)| (id.as_str(), *n))
                .collect::<Vec<_>>(),
            [
                ("p", 1),
                ("s1", 1),
                ("a", 1),
                ("c", 1),
                ("sub", 1),
                ("b", 2),
                ("s2", 2),
                ("d", 3)
            ]
        );
        assert_eq!(
            events[5].2,
            [
                ("part", "p".into(), 1),
                ("chapter", "c".into(), 1),
                ("subsection", "sub".into(), 1)
            ]
        );
        let next: Presentation =
            serde_json::from_value(json!({"size":"L","nodes":[article("x", "h")]})).unwrap();
        walk(&next.nodes, &mut number, &mut |_, n, ancestors| {
            assert_eq!(n, 4);
            assert!(ancestors.is_empty());
            Ok(())
        })
        .unwrap();
    }
}
