use std::fmt::Debug;

    pub fn get_variant_name<T: Debug>(v: &T) -> String {
        format!("{:?}", v).split('(').next().unwrap().to_string()
    }

