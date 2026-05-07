use crate::models::NamesDatabase;
use lazy_static::lazy_static;
use rand::seq::IndexedRandom;
use rand::RngExt;
use std::fs;
use std::path::PathBuf;

lazy_static! {
    pub static ref NAMES_DB: NamesDatabase = load_names_database();
}

/// 加载姓名数据库
///
/// 从多个可能的位置查找并加载 names.json 资源文件：
/// 1. 可执行文件所在目录的 resources/names.json
/// 2. 可执行文件所在目录的上级目录的 resources/names.json
/// 3. 当前工作目录的 resources/names.json
///
/// 如果所有位置都找不到，则返回内置的默认数据。
fn load_names_database() -> NamesDatabase {
    let exe_path = std::env::current_exe().unwrap_or_default();
    let resource_paths = vec![
        exe_path.parent().unwrap().join("resources/names.json"),
        exe_path.parent().unwrap().parent().unwrap().join("resources/names.json"),
        PathBuf::from("resources/names.json"),
    ];

    for path in resource_paths {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(db) = serde_json::from_str::<NamesDatabase>(&content) {
                    return db;
                }
            }
        }
    }

    NamesDatabase {
        chinese_name: crate::models::NameData {
            male: vec!["伟".to_string(), "强".to_string(), "磊".to_string()],
            female: vec!["芳".to_string(), "娜".to_string(), "秀".to_string()],
        },
        western_name: crate::models::NameData {
            male: vec!["James".to_string(), "John".to_string(), "Michael".to_string()],
            female: vec!["Mary".to_string(), "Jennifer".to_string(), "Linda".to_string()],
        },
        chinese_place: vec!["京城".to_string(), "长安".to_string()],
        western_place: vec!["London".to_string(), "Paris".to_string()],
    }
}

/// 生成随机姓名或地名
///
/// 根据类别和性别生成随机姓名或地名：
/// - `chinese_name`: 中文姓名（姓氏+名字）
/// - `western_name`: 西方姓名
/// - `chinese_place`: 中文地名
/// - `western_place`: 西方地名
///
/// # 参数
/// - `category`: 类别（chinese_name, western_name, chinese_place, western_place）
/// - `gender`: 性别筛选（male, female, None 表示混合）
/// - `count`: 生成数量（最大100）
///
/// # 返回值
/// 生成的随机名称列表
#[tauri::command]
pub fn generate_names(category: String, gender: Option<String>, count: u32) -> Vec<String> {
    let mut rng = rand::rng();
    let count = count.min(100) as usize;

    match category.as_str() {
        "chinese_name" => {
            let (surnames, given_names) = match gender.as_deref() {
                Some("male") => (
                    &["王", "李", "张", "刘", "陈", "杨", "赵", "黄", "周", "吴",
                      "徐", "孙", "胡", "朱", "高", "林", "何", "郭", "马", "罗",
                      "梁", "宋", "郑", "谢", "韩", "唐", "冯", "于", "董", "萧"][..],
                    &NAMES_DB.chinese_name.male[..],
                ),
                Some("female") => (
                    &["王", "李", "张", "刘", "陈", "杨", "赵", "黄", "周", "吴",
                      "徐", "孙", "胡", "朱", "高", "林", "何", "郭", "马", "罗",
                      "李", "王", "张", "刘", "陈", "杨", "赵", "黄", "周", "吴"][..],
                    &NAMES_DB.chinese_name.female[..],
                ),
                _ => (
                    &["王", "李", "张", "刘", "陈", "杨", "赵", "黄", "周", "吴",
                      "徐", "孙", "胡", "朱", "高", "林", "何", "郭", "马", "罗",
                      "梁", "宋", "郑", "谢", "韩", "唐", "冯", "于", "董", "萧"][..],
                    if rng.random_bool(0.5) { &NAMES_DB.chinese_name.male[..] } else { &NAMES_DB.chinese_name.female[..] },
                ),
            };
            (0..count)
                .map(|_| {
                    let surname = *surnames.choose(&mut rng).unwrap_or(&"李");
                    let idx1 = rng.random_range(0..given_names.len());
                    let idx2 = rng.random_range(0..given_names.len());
                    format!("{}{}{}", surname, given_names[idx1], given_names[idx2])
                })
                .collect()
        }
        "western_name" => {
            let names = match gender.as_deref() {
                Some("male") => &NAMES_DB.western_name.male[..],
                Some("female") => &NAMES_DB.western_name.female[..],
                _ => {
                    if rng.random_bool(0.5) { &NAMES_DB.western_name.male[..] } else { &NAMES_DB.western_name.female[..] }
                }
            };
            names.sample(&mut rng, count).cloned().collect()
        }
        "chinese_place" => NAMES_DB.chinese_place.sample(&mut rng, count).cloned().collect(),
        "western_place" => NAMES_DB.western_place.sample(&mut rng, count).cloned().collect(),
        _ => vec![],
    }
}
