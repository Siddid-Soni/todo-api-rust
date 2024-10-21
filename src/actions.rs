use diesel::prelude::*;

use crate::models;

type DbError = Box<dyn std::error::Error + Send + Sync>;

pub fn create_task(
    conn: &mut PgConnection,
    title: &str,
    desc: Option<&str>,
    comp: Option<bool>,
) -> Result<models::Task, DbError> {
    use crate::schema::tasks;

    let new_task = models::NewTask {
        title: title.to_string(),
        description: desc.map(String::from),
        completed: comp,
    };

    Ok(diesel::insert_into(tasks::table)
        .values(&new_task)
        .returning(models::Task::as_returning())
        .get_result(conn)?)
}

pub fn find_task_by_id(
    conn: &mut PgConnection,
    task_id: i32,
) -> Result<Option<models::Task>, DbError> {
    use crate::schema::tasks::dsl;

    let task = dsl::tasks
        .find(task_id)
        .first(conn);

    match task {
        Ok(task) => Ok(Some(task)),
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn find_all_tasks(
    conn: &mut PgConnection,
) -> Result<Vec<models::Task>, DbError> {
    use crate::schema::tasks::dsl;

    let tasks = dsl::tasks
        .load::<models::Task>(conn)?;

    Ok(tasks)
}

pub fn update_task(
    conn: &mut PgConnection,
    task_id: i32,
    title: Option<&str>,
    desc: Option<&str>,
    comp: Option<bool>,
) -> Result<Option<models::Task>, DbError> {
    use crate::schema::tasks::dsl;

    let task = dsl::tasks
        .find(task_id)
        .first::<models::Task>(conn);

    match task {
        Ok(task) => {
            let updated_task = models::Task {
                id: task.id,
                title: title.map(String::from).unwrap_or(task.title),
                description: desc.map(String::from).or(task.description),
                completed: comp.unwrap_or(task.completed),
            };
                
            Ok(Some(diesel::update(dsl::tasks.find(task_id))
                .set(&updated_task)
                .returning(models::Task::as_returning())
                .get_result(conn)?))
        },
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn delete_task(
    conn: &mut PgConnection,
    task_id: i32,
) -> Result<Option<models::Task>, DbError> {
    use crate::schema::tasks::dsl;

    let task: Result<models::Task, diesel::result::Error> = dsl::tasks
        .find(task_id)
        .first(conn);

    match task {
        Ok(task) => {
            diesel::delete(dsl::tasks.find(task_id))
                .execute(conn)?;
            Ok(Some(task))
        }
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(e) => Err(Box::new(e)),
    }
}
