use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::{NoTls, Error, Row};

use crate::secrets::get_secret;

pub struct Database {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

#[derive(Debug, Clone)]
pub struct MsgLogs {
    pub msg_id: i64,
    pub guild_id: i64,
    pub channel_id: i64,
    pub author_id: i64,
    pub content: String,
    pub attachments: String,
}

impl Database {
    pub async fn new() -> Result<Self, Error> {
        let manager = PostgresConnectionManager::new_from_stringlike(
            format!("host=localhost user=postgres password={}", get_secret("DB_PW")),
            NoTls,
        ).expect("Failed to create connection manager");

        let pool = Pool::builder().build(manager).await.expect("Failed to build pool");
        Ok(Database { pool })
    }

    pub async fn create_table_msg_logs(&self) -> Result<(), Error> {
        let conn = self.pool.get().await.unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS msg_logs (
                msg_id BIGINT PRIMARY KEY,
                guild_id BIGINT,
                channel_id BIGINT,
                author_id BIGINT,
                content TEXT,
                attachments TEXT
            )", &[],
        ).await?;

        println!("Logs table created");
        Ok(())

    }

    pub async fn create_table_deleted_msgs(&self) -> Result<(), Error> {
        let conn = self.pool.get().await.unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS deleted_msgs (
                msg_id BIGINT PRIMARY KEY,
                guild_id BIGINT,
                channel_id BIGINT,
                author_id BIGINT,
                content TEXT,
                attachments TEXT
            )", &[],
        ).await?;
        Ok(())

    }
    

    pub async fn insert_msg_logs(&self, msg_id: i64, guild_id: i64, channel_id: i64, author_id: i64, content: &str, attachments: Vec<String>) -> Result<(), Error> {
        let mut conn = self.pool.get().await.unwrap();

        

        let attachment_string = match attachments.len() {
            0 => "".to_string(), // Return an empty string if the attachments vector is empty
            _ => attachments.join(","), // Join the attachments into a single string separated by ","
        };
    

        let trans = conn.transaction().await?;

        trans.execute(
            "INSERT INTO msg_logs (msg_id, guild_id, channel_id, author_id, content, attachments) VALUES ($1, $2, $3, $4, $5, $6)",
            &[&msg_id, &guild_id, &channel_id, &author_id, &content, &attachment_string],
        ).await?;

        
        let row_count: i64 = trans.query_one("SELECT COUNT(*) FROM msg_logs", &[]).await?.get(0);

        if row_count > 1000 {
            trans.execute(
                "DELETE FROM msg_logs WHERE msg_id = (SELECT msg_id FROM logs ORDER BY msg_id ASC LIMIT 1)",
                &[],
            ).await?;
        }

        trans.commit().await?;

        Ok(())
    }

    pub async fn insert_deleted_msgs(&self, msg_id: i64, guild_id: i64, channel_id: i64, author_id: i64, content: &str, attachments: Vec<String>) -> Result<(), Error> {
        let mut conn = self.pool.get().await.unwrap();
    
        let attachment_string = match attachments.len() {
            0 => "".to_string(), // Return an empty string if the attachments vector is empty
            _ => attachments.join(","), // Join the attachments into a single string separated by ","
        };
    
        let trans = conn.transaction().await?;
    
        trans.execute(
            "INSERT INTO deleted_msgs (msg_id, guild_id, channel_id, author_id, content, attachments) VALUES ($1, $2, $3, $4, $5, $6)",
            &[&msg_id, &guild_id, &channel_id, &author_id, &content, &attachment_string],
        ).await?;
    
        let row_count: i64 = trans.query_one("SELECT COUNT(*) FROM deleted_msgs", &[]).await?.get(0);
    
        if row_count > 1000 {
            trans.execute(
                "DELETE FROM deleted_msgs WHERE msg_id = (SELECT msg_id FROM deleted_msgs ORDER BY msg_id ASC LIMIT 1)",
                &[],
            ).await?;
        }
    
        trans.commit().await?;
    
        Ok(())
    }



    pub async fn read_logs_by_id(&self, msg_id: i64) -> Result<Vec<MsgLogs>, Error> {
        let conn = self.pool.get().await.unwrap();

        let statement = conn.prepare("SELECT * FROM msg_logs WHERE msg_id = $1").await?;

        let rows = conn.query(&statement, &[&msg_id]).await?;

        let mut msg_logs = Vec::new();

        for row in rows {
            msg_logs.push(parse_msg_logs_record(row)?);
        }

        Ok(msg_logs)
    }

    pub async fn get_last_deleted_msgs(&self, guild_id: i64) -> Result<Vec<MsgLogs>, Error> {
        let conn = self.pool.get().await.unwrap();
    
        let statement = conn
            .prepare("SELECT * FROM deleted_msgs WHERE guild_id = $1 ORDER BY msg_id DESC LIMIT 1")
            .await?;
    
        let rows = conn.query(&statement, &[&guild_id]).await?;
    
        let mut msg_logs = Vec::new();
    
        for row in rows {
            msg_logs.push(parse_msg_logs_record(row)?);
        }
    
        Ok(msg_logs)
    }
    

    pub async fn update_logs_by_id(&self, msg_id: i64, new_content: &str) -> Result<(), Error> {
        let mut conn = self.pool.get().await.unwrap();
        
        let trans = conn.transaction().await?;

        trans.execute(
            "UPDATE msg_logs SET content = $1 WHERE msg_id = $2",
            &[&new_content, &msg_id],
        ).await?;

        trans.commit().await?;

        Ok(())
    }
}

fn parse_msg_logs_record(row: Row) -> Result<MsgLogs, Error> {
    Ok(MsgLogs {
        msg_id: row.try_get(0)?,
        guild_id: row.try_get(1)?,
        channel_id: row.try_get(2)?,
        author_id: row.try_get(3)?,
        content: row.try_get(4)?,
        attachments: row.try_get(5)?,
    })
}
