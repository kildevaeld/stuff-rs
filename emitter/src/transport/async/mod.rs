pub trait Executor {
    fn spawn<F>(&self, future: F);
}
