use crate::task::Task;

pub struct TaskGroup<T: Task> {
    tasks: Vec<T>,
    epochs_run: u64,
}

impl<T: Task> TaskGroup<T> {
    pub fn new(tasks: Vec<T>) -> Self {
        Self {
            tasks,
            epochs_run: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn epochs_run(&self) -> u64 {
        self.epochs_run
    }

    pub fn tasks(&self) -> &[T] {
        &self.tasks
    }

    pub fn tasks_mut(&mut self) -> &mut [T] {
        &mut self.tasks
    }

    pub fn into_tasks(self) -> Vec<T> {
        self.tasks
    }

    pub fn run_one_epoch(&mut self) -> Result<(), T::Error> {
        for task in &mut self.tasks {
            task.evolve_one_epoch()?;
        }
        self.epochs_run += 1;
        Ok(())
    }

    pub fn run_epochs(&mut self, num_epochs: u64) -> Result<(), T::Error> {
        for _ in 0..num_epochs {
            self.run_one_epoch()?;
        }
        Ok(())
    }
}
