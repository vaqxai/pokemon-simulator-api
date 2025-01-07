#[tokio::test]
async fn test_db_handle() {
    use crate::database::DbHandle;

    let db = DbHandle::connect().await;

    if let Err(e) = db {
        panic!("Error: {:?}", e);
    }
}

#[tokio::test]
async fn use_db_handle() {
    use crate::database::DbHandle;
    use neo4rs::Node;

    println!("Creating Handle");

    let db: DbHandle = DbHandle::connect().await.unwrap();

    let mut q_out = db
        .inner
        .execute("MATCH (n:STATUS) WHERE n.id = 1 RETURN n;".into())
        .await
        .unwrap();

    while let Ok(Some(row)) = q_out.next().await {
        let node = row.get::<Node>("n").unwrap();

        let status: String = node.get::<String>("status").unwrap().as_str().to_string();

        assert_eq!(status, "ok");
    }
}

#[tokio::test]
async fn test_db_get() {
    use crate::database::{DbGet, DbRepr};
    use neo4rs::Node;

    struct Status {
        id: i32,
        status: String,
    }

    impl DbRepr for Status {
        const DB_NODE_KIND: &'static str = "STATUS";

        fn get_identifier(&self) -> String {
            self.id.to_string()
        }
    }

    impl DbGet for Status {
        fn from_db_node(node: Node) -> Self::Future {
            Box::pin(async move {
                Ok(Self {
                    id: node.get("id")?,
                    status: node.get("status")?,
                })
            })
        }
    }

    let status = Status::get_first("1").await.unwrap();

    assert_eq!(status.id, 1);
    assert_eq!(status.status, "ok");
}

#[tokio::test]
async fn test_db_repr() {
    use crate::database::DbRepr;

    struct Status;

    impl DbRepr for Status {
        const DB_NODE_KIND: &'static str = "STATUS";

        fn get_identifier(&self) -> String {
            "1".to_string()
        }
    }

    assert_eq!(Status::DB_NODE_KIND, "STATUS");
}

#[tokio::test]
async fn test_db_put() {
    use crate::database::{DbGet, DbPut, DbRepr};
    use neo4rs::Node;

    #[derive(Debug)]
    struct Status {
        id: u32,
        status: String,
    }

    impl DbRepr for Status {
        const DB_NODE_KIND: &'static str = "STATUS";

        fn get_identifier(&self) -> String {
            self.id.to_string()
        }
    }

    impl DbPut for Status {
        fn put_args(&self) -> String {
            format!("{{id: {}, status: '{}'}}", self.id, self.status)
        }
    }

    impl DbGet for Status {
        fn from_db_node(node: Node) -> Self::Future {
            Box::pin(async move {
                Ok(Self {
                    id: node.get("id")?,
                    status: node.get("status")?,
                })
            })
        }
    }

    let status = Status {
        id: u32::MAX,
        status: "ok".to_string(),
    };

    println!("Generated struct: {:?}", status);

    status.put_self().await.unwrap();

    println!("Put self completed");

    // assert this is now in the database
    let status = Status::get_first(&u32::MAX.to_string()).await.unwrap();

    assert_eq!(status.id, u32::MAX);
    assert_eq!(status.status, "ok");

    // delete status from db

    let db = crate::database::DbHandle::connect().await.unwrap();

    let mut q_res = db
        .inner
        .execute(
            format!(
                "MATCH (n:{}) WHERE n.id = {} DELETE n;",
                Status::DB_NODE_KIND,
                u32::MAX
            )
            .into(),
        )
        .await
        .unwrap();

    let _none = q_res.next().await.unwrap();

    // assert this is no longer in the database
    let status = Status::get_first(&u32::MAX.to_string()).await;

    assert!(status.is_err());
}

#[tokio::test]
async fn test_db_delete() {
    use crate::database::{DbDelete, DbGet, DbPut, DbRepr};
    use neo4rs::Node;

    struct Status {
        id: u32,
        status: String,
    }

    impl DbPut for Status {
        fn put_args(&self) -> String {
            format!("{{id: {}, status: '{}'}}", self.id, self.status)
        }
    }

    impl DbRepr for Status {
        const DB_NODE_KIND: &'static str = "STATUS";

        fn get_identifier(&self) -> String {
            self.id.to_string()
        }
    }

    impl DbDelete for Status {}

    impl DbGet for Status {
        fn from_db_node(node: Node) -> Self::Future {
            Box::pin(async move {
                Ok(Self {
                    id: node.get("id")?,
                    status: node.get("status")?,
                })
            })
        }
    }

    let status = Status {
        id: u32::MAX - 1,
        status: "ok".to_string(),
    };

    let identifier = status.id.to_string();

    status.put_self().await.unwrap();

    Status::delete(&identifier).await.unwrap();

    let status = Status::get_first(&identifier);

    assert!(status.await.is_err());
}

#[tokio::test]
async fn test_db_update() {
    use crate::database::{DbDelete, DbGet, DbPut, DbRepr, DbUpdate};
    use neo4rs::Node;

    struct Status {
        id: u32,
        status: String,
    }

    impl DbPut for Status {
        fn put_args(&self) -> String {
            format!("{{id: {}, status: '{}'}}", self.id, self.status)
        }
    }

    impl DbRepr for Status {
        const DB_NODE_KIND: &'static str = "STATUS";

        fn get_identifier(&self) -> String {
            self.id.to_string()
        }
    }

    impl DbGet for Status {
        fn from_db_node(node: Node) -> Self::Future {
            Box::pin(async move {
                Ok(Self {
                    id: node.get("id")?,
                    status: node.get("status")?,
                })
            })
        }
    }

    impl DbUpdate for Status {
        fn update_args(&self) -> String {
            format!("n.status = '{}'", self.status)
        }
    }

    impl DbDelete for Status {}

    let status = Status {
        id: u32::MAX - 2,
        status: "ok".to_string(),
    };

    let identifier = status.id.to_string();

    status.put_self().await.unwrap();

    let mut status = Status::get_first(&identifier).await.unwrap();

    assert_eq!(status.status, "ok");

    status.status = "not ok".to_string();

    status.update(&identifier).await.unwrap();

    let status = Status::get_first(&identifier).await.unwrap();

    assert_eq!(status.status, "not ok");

    Status::delete(&identifier).await.unwrap();

    let status = Status::get_first(&identifier);

    assert!(status.await.is_err());
}
