// 用户配置相关命令

// use crate::AppState;
// use sea_orm::{ActiveModelTrait, EntityTrait, Set};
// use tauri::State;

// use crate::database::entities::{prelude::UserConfig, user_config};

// /// 获取用户配置
// #[tauri::command]
// pub async fn get_user_config(state: State<'_, AppState>) -> Result<user_config::Model, String> {
//     let db = state.db.as_ref();

//     // 尝试获取用户配置，如果不存在则创建默认配置
//     let user_config = UserConfig::find_by_id("default")
//         .one(db)
//         .await
//         .map_err(|e| e.to_string())?;

//     match user_config {
//         Some(config) => Ok(config),
//         None => {
//             // 创建默认用户配置
//             let now = chrono::Utc::now().timestamp();
//             let new_config = user_config::ActiveModel {
//                 username: Set("default".to_string()),
//                 email: Set("default@example.com".to_string()),
//                 student_num: Set("202501001".to_string()),
//                 phone: Set(None),
//                 avatar: Set(None),
//                 theme: Set(Some("light".to_string())),
//                 password_hash: Set(None),
//                 ai_base_url: Set(None),
//                 ai_key: Set(None),
//                 sync: Set(false),
//                 created_at: Set(now),
//                 updated_at: Set(now),
//             };

//             let _ = new_config.insert(db).await;

//             // 再次获取配置
//             let config = UserConfig::find_by_id("default")
//                 .one(db)
//                 .await
//                 .map_err(|e| e.to_string())?
//                 .ok_or("Failed to create user config".to_string())?;

//             Ok(config)
//         }
//     }
// }

// /// 更新用户配置
// #[tauri::command]
// pub async fn update_user_config(
//     state: State<'_, AppState>,
//     ai_base_url: Option<String>,
//     ai_key: Option<String>,
// ) -> Result<user_config::Model, String> {
//     let db = state.db.as_ref();

//     let user_config = UserConfig::find_by_id("default")
//         .one(db)
//         .await
//         .map_err(|e| e.to_string())?
//         .ok_or("User config not found".to_string())?;

//     let mut config: user_config::ActiveModel = user_config.into();
//     let now = chrono::Utc::now().timestamp();

//     if let Some(ai_base_url) = ai_base_url {
//         config.ai_base_url = Set(Some(ai_base_url));
//     }

//     if let Some(ai_key) = ai_key {
//         config.ai_key = Set(Some(ai_key));
//     }

//     config.updated_at = Set(now);

//     let updated_config = config.update(db).await.map_err(|e| e.to_string())?;

//     Ok(updated_config)
// }
