use uuid::Uuid;

pub fn uuid_v4_str() -> String {
    let uuid = Uuid::new_v4();
    uuid.to_string()
}
