use crate::interface::event_system::event::Event;

#[derive(Clone)]
pub struct TaskCreateEvent {}

impl Event for TaskCreateEvent {}

#[derive(Clone)]
pub struct TaskStartEvent {}

impl Event for TaskStartEvent {}

#[derive(Clone)]
pub struct TaskPauseEvent {}

impl Event for TaskPauseEvent {}

#[derive(Clone)]
pub struct TaskResumeEvent {}

impl Event for TaskResumeEvent {}

#[derive(Clone)]
pub struct TaskCompleteEvent {}
impl Event for TaskCompleteEvent {}

#[derive(Clone)]
pub struct TaskCancelEvent {}

impl Event for TaskCancelEvent {}

#[derive(Clone)]
pub struct TaskFailEvent {}

impl Event for TaskFailEvent {}
