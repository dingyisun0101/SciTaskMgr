use super::Task;

/// Build one task from one owned config value.
pub fn build_task<T>(config: T::Config) -> Result<T, T::Error>
where
    T: Task,
{
    T::new(config)
}

/// Build `num_tasks` identical tasks by cloning one config value.
pub fn build_task_copies<T>(config: T::Config, num_tasks: usize) -> Result<Vec<T>, T::Error>
where
    T: Task,
{
    let mut tasks = Vec::with_capacity(num_tasks);
    for _ in 0..num_tasks {
        tasks.push(T::new(config.clone())?);
    }
    Ok(tasks)
}

/// Build one task per config from a list of distinct config values.
pub fn build_tasks_from_configs<T>(configs: Vec<T::Config>) -> Result<Vec<T>, T::Error>
where
    T: Task,
{
    let mut tasks = Vec::with_capacity(configs.len());
    for config in configs {
        tasks.push(T::new(config)?);
    }
    Ok(tasks)
}
