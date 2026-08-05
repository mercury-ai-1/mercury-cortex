use mercury_cortex_core::db;
use mercury_cortex_core::schema;

#[tokio::test]
async fn probe_info_shapes() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("probe.db");

    {
        let db = db::initialize(&db_path).await.unwrap();
        schema::run_pending(&db).await.unwrap();

        db.query("CREATE users:agent1 SET name = 'Agent', email = 'a@b.c', agent_name = 'agent-one', created_at = time::now(), updated_at = time::now()").await.unwrap();
        db.query("CREATE projects:p1 SET owner_id = users:agent1, name = 'P1', slug = 'p1', root_path = '/tmp/p1', created_at = time::now(), updated_at = time::now()").await.unwrap();
        db.query("CREATE projects:p2 SET owner_id = users:agent1, name = 'P2', slug = 'p2', root_path = '/tmp/p2', created_at = time::now(), updated_at = time::now()").await.unwrap();
        db.query("CREATE file_data:f1 SET project_id = projects:p1, path = '/a.rs', indexed_at = time::now(), updated_at = time::now()").await.unwrap();
        db.query("CREATE file_data:f2 SET project_id = projects:p1, path = '/b.rs', indexed_at = time::now(), updated_at = time::now()").await.unwrap();
        db.query("CREATE file_data:f3 SET project_id = projects:p2, path = '/c.rs', indexed_at = time::now(), updated_at = time::now()").await.unwrap();

        let info: Vec<serde_json::Value> = db.query("INFO FOR DB").await.unwrap().take(0).unwrap();
        let tables = info[0]["tables"].as_object().unwrap();
        eprintln!("=== tables (keys) ===");
        for k in tables.keys() {
            eprintln!("  {k}");
        }

        let tbl: Vec<serde_json::Value> = db
            .query("INFO FOR TABLE file_data")
            .await
            .unwrap()
            .take(0)
            .unwrap();
        let fields = tbl[0]["fields"].as_object().unwrap();
        eprintln!("=== file_data fields (keys) ===");
        for k in fields.keys() {
            eprintln!("  {k}");
        }
        eprintln!(
            "file_data has project_id: {}",
            fields.contains_key("project_id")
        );

        let tbl2: Vec<serde_json::Value> = db
            .query("INFO FOR TABLE users")
            .await
            .unwrap()
            .take(0)
            .unwrap();
        let fields2 = tbl2[0]["fields"].as_object().unwrap();
        eprintln!(
            "users has project_id: {}",
            fields2.contains_key("project_id")
        );

        let pid = mercury_cortex_core::util::project_id_value("projects:p1").unwrap();
        let rows: Vec<serde_json::Value> = db
            .query("SELECT * FROM file_data WHERE project_id = $pid")
            .bind(("pid", pid))
            .await
            .unwrap()
            .take(0)
            .unwrap();
        eprintln!("=== filtered SELECT file_data WHERE project_id = projects:p1 ===");
        eprintln!("{}", serde_json::to_string_pretty(&rows).unwrap());
    }

    for _ in 0..50 {
        if !db::lock_is_held(dir.path()).unwrap() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    eprintln!(
        "lock_is_held after wait: {}",
        db::lock_is_held(dir.path()).unwrap()
    );
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    eprintln!(
        "lock_is_held after extra 1s: {}",
        db::lock_is_held(dir.path()).unwrap()
    );
    let db = db::initialize(&db_path).await.unwrap();
    let sel: Vec<serde_json::Value> = db
        .query("SELECT * FROM projects")
        .await
        .unwrap()
        .take(0)
        .unwrap();
    eprintln!("reopen SELECT projects: {} rows", sel.len());
}
