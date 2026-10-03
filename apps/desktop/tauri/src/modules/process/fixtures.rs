//! FIX-03 虚拟进程 fixture。
//!
//! 测试只驱动内存中的命令记录，不创建真实子进程，也不访问真实系统资源。

use std::cell::RefCell;
use std::rc::Rc;

use super::{LifecycleResult, ProcessCommand, ProcessRunner};
use crate::errors::AppError;

/// 虚拟进程生命周期动作，用于断言调用顺序与副作用。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessAction {
    Start,
    Stop,
}

/// 测试专用进程运行器，把生命周期调用记录在内存里。
#[derive(Debug, Default)]
pub struct VirtualProcessRunner {
    actions: Rc<RefCell<Vec<ProcessAction>>>,
}

impl VirtualProcessRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute_fixture(&self, _command: &ProcessCommand) -> Result<LifecycleResult, AppError> {
        Ok(LifecycleResult::Started)
    }

    pub fn start(&self) {
        self.actions.borrow_mut().push(ProcessAction::Start);
    }

    pub fn stop(&self) {
        self.actions.borrow_mut().push(ProcessAction::Stop);
    }

    pub fn actions(&self) -> Vec<ProcessAction> {
        self.actions.borrow().clone()
    }

    pub fn clear(&self) {
        self.actions.borrow_mut().clear();
    }
}

impl ProcessRunner for VirtualProcessRunner {
    fn execute(&self, command: &ProcessCommand) -> Result<LifecycleResult, AppError> {
        command.validate()?;
        self.execute_fixture(command)
    }

    fn cancel_pending(&self) -> Result<(), AppError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_runner_records_start_and_stop_without_real_processes() {
        let runner = VirtualProcessRunner::new();

        runner.start();
        runner.stop();

        assert_eq!(
            runner.actions(),
            vec![ProcessAction::Start, ProcessAction::Stop]
        );

        runner.clear();
        assert!(runner.actions().is_empty());
    }
}
