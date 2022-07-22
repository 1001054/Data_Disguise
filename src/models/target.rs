use serde::Serialize;
/// the object in the web's database
/// which could be user's info or the publications
#[derive(Serialize, Debug, Clone)]
pub struct Target {
    pub primary_key_index: Option<usize>,
    pub fields: Option<Vec<Field>>
}
impl Target {
    /// get the field of primary key
    pub fn primary_key(&self) -> Option<Field> {
        let index = self.primary_key_index.unwrap();
        let temp = self.fields.as_ref().unwrap();
        if index >= temp.len() {
            None
        } else {
            Some(temp[index].clone())
        }
    }
    /// get the field of the foreign key by its field name
    pub fn foreign_key(&self, foreign_key_name: &str) -> Option<Field> {
        let mut res = None;
        for field in self.fields.as_ref().unwrap() {
            if field.field_name.as_ref().unwrap().eq(foreign_key_name) {
                res = Some(field.clone());
                break
            }
        }
        res
    }
    /// get all the fields' names seperated by ", "
    pub fn field_names(&self) -> Option<String> {
        let mut res = String::new();
        match &self.fields {
            None => {
                None
            }
            Some(fields) => {
                for field in fields {
                    res = res + ", " + field.field_name.as_ref().unwrap().as_str();
                }
                Some(res[2..].to_string())
            }
        }
    }
    /// get all the fields' values seperated by ", "
    pub fn field_values(&self) -> Option<String> {
        let mut res = String::new();
        match &self.fields {
            None => {
                None
            }
            Some(fields) => {
                for field in fields {
                    let mut temp = String::new();
                    if field.field_type.as_ref().unwrap().contains("int") {
                        temp.push_str(field.field_value.as_ref().unwrap().as_str());
                    }else {
                        temp.push_str("\"");
                        temp.push_str(field.field_value.as_ref().unwrap().as_str());
                        temp.push_str("\"");
                    }
                    res.push_str(", ");
                    res.push_str(temp.as_str());
                }
                Some(res[2..].to_string())
            }
        }
    }
}
/// the field of the web' database table (object's field)
#[derive(Serialize, Debug, Clone)]
pub struct Field {
    pub field_name: Option<String>,
    pub field_type: Option<String>,
    pub field_value: Option<String>
}