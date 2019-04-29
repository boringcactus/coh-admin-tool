use actix::prelude::*;
use odbc;
pub use odbc::Result;

pub struct CreateUser {
    pub username: String,
    pub password: String,
}

impl Message for CreateUser {
    type Result = odbc::Result<()>;
}

pub struct Database {
    env: odbc::Environment<odbc::Version3>,
}

impl Actor for Database {
    type Context = SyncContext<Self>;
}

impl Database {
    pub fn new() -> Result<Database> {
        let env = odbc::create_environment_v3().map_err(|e| e.unwrap())?;
        Ok(Database {
            env,
        })
    }

    fn connect(&self) -> Result<odbc::Connection> {
        self.env.connect_with_connection_string("FILEDSN=AuthDB.dsn;")
    }
}

fn adler32(data: &str) -> u32 {
    let mod_adler = 65521;
    let mut a = 1;
    let mut b = 0;
    for ch in data.chars() {
        let ch: u32 = ch.into();
        a = (a + ch) % mod_adler;
        b = (b + a) % mod_adler;
    }
    (b << 16) | a
}

fn hash_password(username: &str, password: &str) -> String {
    use sha2::{Sha512, Digest};
    let username = username.to_lowercase();
    let a32 = adler32(&username);
    let a32hex = format!("{:08x}", a32);
    let a32hex = a32hex[6..8].to_owned() + &a32hex[4..6] + &a32hex[2..4] + &a32hex[0..2];
    let sha = Sha512::default().chain(password).chain(a32hex).result();
    format!("{:x}", sha)
}

impl Handler<CreateUser> for Database {
    type Result = odbc::Result<()>;

    fn handle(&mut self, msg: CreateUser, _: &mut Self::Context) -> Self::Result {
        let conn = self.connect()?;

        let ref username = msg.username;
        let ref password = msg.password;

        let password = hash_password(username, password);

        let uid = {
            let stmt = odbc::Statement::with_parent(&conn)?;
            let sql = "SELECT MAX(uid) + 1 FROM cohauth.dbo.user_account";
            match stmt.exec_direct(sql)? {
                odbc::Data(mut stmt) => {
                    let mut cursor = stmt.fetch()?.expect("Failed to get next user ID");
                    cursor.get_data::<i64>(1)?.expect("Failed to get next user ID")
                }
                odbc::NoData(_) => panic!("Failed to get next user ID"),
            }
        };

        // user_account
        {
            let stmt = odbc::Statement::with_parent(&conn)?;
            let stmt = stmt.bind_parameter(1, username)?;
            let stmt = stmt.bind_parameter(2, &uid)?;
            let stmt = stmt.bind_parameter(3, &uid)?;
            let sql = "INSERT INTO cohauth.dbo.user_account (account, uid, forum_id, pay_stat) VALUES (?, ?, ?, 1014)";
            match stmt.exec_direct(sql)? {
                odbc::NoData(_) => {} // things worked
                odbc::Data(_) => {
                    panic!("insert returned data");
                }
            }
        }

        // user_auth
        {
            let stmt = odbc::Statement::with_parent(&conn)?;
            let stmt = stmt.bind_parameter(1, username)?;
            let stmt = stmt.bind_parameter(2, &password)?;
            let sql = "INSERT INTO cohauth.dbo.user_auth (account, password, salt, hash_type) VALUES (?, CONVERT(BINARY(128), ?), 0, 1);";
            match stmt.exec_direct(sql)? {
                odbc::NoData(_) => {} // things worked
                odbc::Data(_) => {
                    panic!("insert returned data");
                }
            }
        }

        // user_data
        {
            let stmt = odbc::Statement::with_parent(&conn)?;
            let stmt = stmt.bind_parameter(1, &uid)?;
            let sql = "INSERT INTO cohauth.dbo.user_data (uid, user_data) VALUES (?, 0x0080C2E000D00B0C000000000CB40058);";
            match stmt.exec_direct(sql)? {
                odbc::NoData(_) => {} // things worked
                odbc::Data(_) => {
                    panic!("insert returned data");
                }
            }
        }

        // user_server_group
        {
            let stmt = odbc::Statement::with_parent(&conn)?;
            let stmt = stmt.bind_parameter(1, &uid)?;
            let sql = "INSERT INTO cohauth.dbo.user_server_group (uid, server_group_id) VALUES (?, 1);";
            match stmt.exec_direct(sql)? {
                odbc::NoData(_) => {} // things worked
                odbc::Data(_) => {
                    panic!("insert returned data");
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn known_good_hash() {
        assert_eq!(super::hash_password("test", "password"), "46ffce3efcfe83bfa205b076d7c2084b9dcf04cdb26f9019103cde29779d26a85216b2c0f43ba1a8fb9b7fa22f05a949bf4edc314af27629e8fc23014e77a24d")
    }
}
