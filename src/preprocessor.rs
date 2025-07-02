use regex::Regex;

pub fn interpolate_strings(text: &str) -> String {
    let re_string = Regex::new(r#""((?:[^"\\]|\\.)*)""#).unwrap();
    let re_interpolation = Regex::new(r#"\$\{(.*?)}"#).unwrap();

    let res = re_string.replace_all(text, |str_cap: &regex::Captures| {
        let string_val = &str_cap[1];
        let new_string = re_interpolation.replace_all(string_val, |var_cap: &regex::Captures| {
            let var_name = &var_cap[1];
            format!("\" .. tostring({}) .. \"", var_name)
        });
        format!(r#""{}""#, new_string)
    });

    res.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn interpolate_strings_test() {
        let source = r#"
local a = "this is not going to be replaced"
local b = "this is going to be ${found} because it has the special pattern"
local c = "this \"might\" fail and it has ${another} too"
"#;

        let expected = r#"
local a = "this is not going to be replaced"
local b = "this is going to be " .. tostring(found) .. " because it has the special pattern"
local c = "this \"might\" fail and it has " .. tostring(another) .. " too"
"#;
        
        let res = interpolate_strings(source);
        assert_eq!(res, expected);
    }
    
    #[test]
    fn interpolate_strings_start_end() {
        let source = r#""${var} in the start""#;
        let res = interpolate_strings(source);
        assert_eq!(res, r#""" .. tostring(var) .. " in the start""#);

        let source = r#""now in the ${end}""#;
        let res = interpolate_strings(source);
        assert_eq!(res, r#""now in the " .. tostring(end) .. """#);
    }
}