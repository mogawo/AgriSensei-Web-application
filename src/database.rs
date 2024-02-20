
use core::panic;
use std::fmt::format;
pub use std::fs::remove_file;
use std::fs::File;
use std::fs::OpenOptions;
pub use std::io::ErrorKind as IOError;
pub use std::io::ErrorKind::NotFound;
pub use std::fmt::{Display, Debug};

pub use rusqlite::{named_params, params, Connection};
pub use rusqlite::Error as SQLError;

pub use crate::components::*;

pub type Result<T> = std::result::Result<T, DBError>;

#[derive(Debug)]
pub enum DBError{
    SQLError(SQLError),
    IOError(IOError),
}
impl Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = 100;
        match self{
            
            DBError::SQLError(err) => write!(f, "{}" , format!(">{:=^width$}<\n{:?}", "[SQL Error]", err)),
            DBError::IOError(err) => write!(f, "{}", format!(">{:=^width$}<\n{:?}", "[IO Error]", err)) 
        }
    }
}

impl std::error::Error for DBError{}

impl From<SQLError> for DBError{
    fn from(value: SQLError) -> Self {
        DBError::SQLError(value)
    }
}

impl From<IOError> for DBError{
    fn from(value: IOError) -> Self {
        DBError::IOError(value)
    }
}



pub struct TableColumnNames{}
impl TableColumnNames{
    //User Table
    pub const USERS: &'static str = r"users";
    pub const USER_ID: &'static str = r"userID";
    pub const USER_NAME: &'static str = r"userName";
    
    //Sensor Table
    pub const SENSORS: &'static str = r"sensors";
    pub const SENSOR_ID: &'static str = r"sensorID";
    pub const SENSOR_TYPE: &'static str = r"sensorType";
    
    //Data Packet
    pub const DATA_PACKET: &'static str = r"dataPacket";
    pub const DATE_TIME: &'static str = r"dateTime";
    pub const SAMPLE_FREQUENCY: &'static str = r"sampleFrequency";
    pub const SAMPLE_DURATION: &'static str = r"sampleDuration";
    pub const SAMPLE_AMOUNT: &'static str = r"sampleAmount";
}

use TableColumnNames as Col;

use crate::components;


pub struct Database{}



impl<'d> Database{
    const DB_NAME: &'static str = "AgriSensei Database";
    const DB_PATH: &'static str = r"data\agrisensei.db";

    pub fn connect() -> Connection{
        match Connection::open(Database::DB_PATH){
            Ok(conn) => conn,
            Err(_) => panic!("Can not connect to {0} @path=...\\{1}", Database::DB_NAME, Database::DB_PATH)
        }
    }

    pub fn new(){
        let _ = remove_file(Database::DB_PATH);
        //std::fs::write(Database::DB_PATH, "");
        
        let users_table = format!(
            "CREATE TABLE IF NOT EXISTS  [{users}](
                [{userID}]  INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT, 
                [{userName}] TEXT DEFAULT 'John Smith'
              );",
              users=Col::USERS,
              userID=Col::USER_ID,
              userName=Col::USER_NAME,);
              
        let sensor_table = format!("CREATE TABLE IF NOT EXISTS [{sensors}](
            [{sensorID}] INTEGER NOT NULL PRIMARY KEY,
            [{sensorType}] TEXT NOT NULL,
            [{userID}] INTEGER NOT NULL,
            FOREIGN KEY ({userID}) REFERENCES {users} ({userID})
          );",
          sensors=Col::SENSORS,
          sensorID=Col::SENSOR_ID,
          sensorType=Col::SENSOR_TYPE,
          users=Col::USERS,
          userID=Col::USER_ID,);
              
             let packet_table = format!("CREATE TABLE IF NOT EXISTS [{dataPacket}](
                [{dateTime}] TEXT NOT NULL PRIMARY KEY,
                [{sampleFrequency}] INTEGER NOT NULL,
                [{sampleDuration}] INTEGER NOT NULL,
                [{sampleAmount}] INTEGER NOT NULL,
                [{sensorID}] INTEGER NOT NULL,
                FOREIGN KEY ({userID}) REFERENCES {users} ({userID})
                FOREIGN KEY ({sensorID}) REFERENCES {sensors} ({sensorID})
              );",
              dataPacket=Col::DATA_PACKET,
              dateTime=Col::DATE_TIME,
              sampleFrequency=Col::SAMPLE_FREQUENCY,
              sampleDuration=Col::SAMPLE_DURATION,
              sampleAmount=Col::SAMPLE_AMOUNT,
              users=Col::USERS,
              userID=Col::USER_ID,
              sensors=Col::SENSORS,
              sensorID=Col::SENSOR_ID);
              

        let conn = Database::connect();
        conn.execute(&users_table, ()).unwrap();
        conn.execute(&sensor_table, ()).unwrap();
        conn.execute(&packet_table, ()).unwrap();
    }
    //userId auto increments in sqlite
    pub fn new_user(name: &str) -> Option<u64>{
        let conn = Database::connect();
        let user_insert = format!(r"INSERT INTO {user}({userName}) VALUES (?1)", user=Col::USERS, userName=Col::USER_NAME);
        match conn.execute(user_insert.as_str(), params![name]){
                Ok(0) => None,
                Ok(_) => {
                    let user_id = conn.last_insert_rowid().try_into().unwrap();
                    println!("Inserted New User with ID={user_id}"); 
                    Some(user_id)
                },
                Err(err) => panic!("[New User] Bad SQL Insert in Database.rs")
            }
    }

    pub fn new_sensor(sensor_type: SensorType, user_id: u64) -> Option<u64>{
        let conn = Database::connect();
        let sensor_insert = format!(r"INSERT INTO {sensors}({sensorType}, {userID}) VALUES (?1, ?2)", sensors=Col::SENSORS, sensorType=Col::SENSOR_TYPE, userID=Col::USER_ID);
        match conn.execute(&sensor_insert, params![sensor_type, user_id]){
            Ok(0) => None,
            Ok(_) => {
                let sensor_id = conn.last_insert_rowid().try_into().unwrap();
                println!("Inserted New Sensor#{sensor_id} for User#{user_id}"); 
                Some(sensor_id)
            }
            Err(e)  => panic!("[New Sensor] Bad SQL Insert")
        }
    }

    pub fn add_packet(packets: DataPacket){
        let conn = Database::connect();
        let packet_insert = format!("INSERT INTO {dataPacket}({dateTime}, {samFreq}, {samDur}, {samAmnt}, {userID}, {sensorID}) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", 
            dataPacket = Col::DATA_PACKET,
            dateTime   = Col::DATE_TIME,
            samFreq    = Col::SAMPLE_FREQUENCY,
            samDur     = Col::SAMPLE_DURATION,
            samAmnt    = Col::SAMPLE_AMOUNT,
            userID     = Col::USER_ID,
            sensorID   = Col::SENSOR_ID
        );
        
        match conn.execute(&packet_insert, //Args need to match order in SQL Table Column index
            params![packets.date_time,
                    packets.frequency,
                    packets.duration,
                    packets.amount,
                    packets.sensor_id]){
            Ok(0) => {print!("Same Packet was Inputted, packet was stored"); ()}
            Ok(rows_inserted) => print!("{rows_inserted} was inserted into DataPackets Table"),
            Err(_e) => panic!("[Add Packet] Bad SQL Insert"),
        }
    }
}