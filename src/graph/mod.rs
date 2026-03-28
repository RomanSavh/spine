use serde::Serialize;
use utoipa::ToSchema;

use crate::models::Service;

#[derive(Debug, Clone, Serialize, PartialEq, ToSchema)]
pub struct DependencyGraph {
    pub service: String,
    pub depends_on: Vec<Dependency>,
    pub depended_by: Vec<Dependency>,
}

#[derive(Debug, Clone, Serialize, PartialEq, ToSchema)]
pub struct Dependency {
    pub service: String,
    pub relation: String,
    pub via: String,
}

pub fn compute_dependencies(target: &str, all_services: &[Service]) -> Option<DependencyGraph> {
    let target_svc = all_services.iter().find(|s| s.name == target)?;

    let mut depends_on = Vec::new();
    let mut depended_by = Vec::new();

    for other in all_services {
        if other.name == target {
            continue;
        }

        // target's grpc_clients → find who serves them
        for client_name in &target_svc.grpc_clients {
            if other.grpc_servers.contains(client_name) {
                depends_on.push(Dependency {
                    service: other.name.clone(),
                    relation: "grpc".to_string(),
                    via: client_name.clone(),
                });
            }
        }

        // target's http_clients → find who serves them
        for http_client in &target_svc.http_clients {
            if other.name == *http_client && other.http_server {
                depends_on.push(Dependency {
                    service: other.name.clone(),
                    relation: "http".to_string(),
                    via: http_client.clone(),
                });
            }
        }

        // target subscribes to queues → find who publishes them
        for queue_name in &target_svc.queue_subscribers {
            if other.queue_publishers.contains(queue_name) {
                depends_on.push(Dependency {
                    service: other.name.clone(),
                    relation: "queue".to_string(),
                    via: queue_name.clone(),
                });
            }
        }

        // other's grpc_clients → if target serves them
        for client_name in &other.grpc_clients {
            if target_svc.grpc_servers.contains(client_name) {
                depended_by.push(Dependency {
                    service: other.name.clone(),
                    relation: "grpc".to_string(),
                    via: client_name.clone(),
                });
            }
        }

        // other's http_clients → if target is one
        for http_client in &other.http_clients {
            if *http_client == target && target_svc.http_server {
                depended_by.push(Dependency {
                    service: other.name.clone(),
                    relation: "http".to_string(),
                    via: http_client.clone(),
                });
            }
        }

        // other subscribes to queues target publishes
        for queue_name in &other.queue_subscribers {
            if target_svc.queue_publishers.contains(queue_name) {
                depended_by.push(Dependency {
                    service: other.name.clone(),
                    relation: "queue".to_string(),
                    via: queue_name.clone(),
                });
            }
        }
    }

    Some(DependencyGraph {
        service: target.to_string(),
        depends_on,
        depended_by,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_service(name: &str) -> Service {
        Service {
            name: name.to_string(),
            description: String::new(),
            github_repo: None,
            grpc_servers: vec![],
            grpc_clients: vec![],
            http_server: false,
            http_clients: vec![],
            queue_publishers: vec![],
            queue_subscribers: vec![],
            tables: vec![],
        }
    }

    #[test]
    fn test_grpc_dependency() {
        let mut a = make_service("svc-a");
        a.grpc_clients = vec!["SvcB".to_string()];
        let mut b = make_service("svc-b");
        b.grpc_servers = vec!["SvcB".to_string()];

        let graph = compute_dependencies("svc-a", &[a, b]).unwrap();
        assert_eq!(graph.depends_on.len(), 1);
        assert_eq!(graph.depends_on[0].service, "svc-b");
        assert_eq!(graph.depends_on[0].relation, "grpc");
        assert_eq!(graph.depends_on[0].via, "SvcB");
    }

    #[test]
    fn test_reverse_grpc_dependency() {
        let mut a = make_service("svc-a");
        a.grpc_clients = vec!["SvcB".to_string()];
        let mut b = make_service("svc-b");
        b.grpc_servers = vec!["SvcB".to_string()];

        let graph = compute_dependencies("svc-b", &[a, b]).unwrap();
        assert_eq!(graph.depended_by.len(), 1);
        assert_eq!(graph.depended_by[0].service, "svc-a");
    }

    #[test]
    fn test_queue_dependency() {
        let mut a = make_service("svc-a");
        a.queue_subscribers = vec!["q1".to_string()];
        let mut b = make_service("svc-b");
        b.queue_publishers = vec!["q1".to_string()];

        let graph = compute_dependencies("svc-a", &[a, b]).unwrap();
        assert_eq!(graph.depends_on.len(), 1);
        assert_eq!(graph.depends_on[0].relation, "queue");
        assert_eq!(graph.depends_on[0].via, "q1");
    }

    #[test]
    fn test_http_dependency() {
        let mut a = make_service("svc-a");
        a.http_clients = vec!["svc-b".to_string()];
        let mut b = make_service("svc-b");
        b.http_server = true;

        let graph = compute_dependencies("svc-a", &[a, b]).unwrap();
        assert_eq!(graph.depends_on.len(), 1);
        assert_eq!(graph.depends_on[0].relation, "http");
    }

    #[test]
    fn test_no_dependencies() {
        let a = make_service("svc-a");
        let b = make_service("svc-b");

        let graph = compute_dependencies("svc-a", &[a, b]).unwrap();
        assert!(graph.depends_on.is_empty());
        assert!(graph.depended_by.is_empty());
    }

    #[test]
    fn test_multiple_dependency_types() {
        let mut a = make_service("svc-a");
        a.grpc_clients = vec!["SvcB".to_string()];
        a.queue_subscribers = vec!["q1".to_string()];
        a.http_clients = vec!["svc-b".to_string()];

        let mut b = make_service("svc-b");
        b.grpc_servers = vec!["SvcB".to_string()];
        b.queue_publishers = vec!["q1".to_string()];
        b.http_server = true;

        let graph = compute_dependencies("svc-a", &[a, b]).unwrap();
        assert_eq!(graph.depends_on.len(), 3);
        let relations: Vec<&str> = graph.depends_on.iter().map(|d| d.relation.as_str()).collect();
        assert!(relations.contains(&"grpc"));
        assert!(relations.contains(&"queue"));
        assert!(relations.contains(&"http"));
    }

    #[test]
    fn test_unknown_service_returns_none() {
        let a = make_service("svc-a");
        assert!(compute_dependencies("nonexistent", &[a]).is_none());
    }
}
