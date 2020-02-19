use
{
	crate          :: { import::*             } ,
	std            :: { rc::Rc, cell::RefCell } ,
};



/// An executor that uses a [tokio::runtime::Runtime] with the [basic scheduler](tokio::runtime::Builder::basic_scheduler).
/// Can spawn `!Send` futures.
///
/// You must make sure that calls to `spawn` and `spawn_local` happen withing a future running on [TokioCt::block_on].
///
/// One feature from tokio is not implemented here, namely the possibility to get a handle that can be sent to another
/// thread to spawn tasks on this executor.
///
/// ## Unwind Safety.
///
/// You must only spawn futures to this API that are unwind safe. Tokio will wrap it in
/// [std::panic::AssertUnwindSafe] and wrap the poll invocation with [std::panic::catch_unwind].
///
/// They reason that this is fine because they require `Send + 'static` on the future. As far
/// as I can tell this is wrong. Unwind safety can be circumvented in several ways even with
/// `Send + 'static` (eg. parking_lot::Mutex is Send + 'static but !UnwindSafe).
///
/// __As added foot gun in the `LocalSpawn` impl for TokioCt we artificially add a Send
/// impl to your future so it can be spawned by tokio, which requires `Send` even for the
/// basic scheduler. This opens more ways to observe broken invariants, like `RefCell`, `TLS`, etc.__
///
/// You should make sure that if your future panics, no code that lives on after the spawned task has
/// unwound, nor any destructors called during the unwind can observe data in an inconsistent state.
///
/// See the relevant [catch_unwind RFC](https://github.com/rust-lang/rfcs/blob/master/text/1236-stabilize-catch-panic.md)
/// and it's discussion threads for more info as well as the documentation in stdlib.
//
#[ derive( Debug, Clone ) ]
//
#[ cfg_attr( feature = "docs", doc(cfg( feature = "tokio_ct" )) ) ]
//
pub struct TokioCt
{
	pub(crate) exec  : Rc<RefCell< Runtime >> ,
	pub(crate) handle: TokioRtHandle          ,
}



impl TokioCt
{
	/// This is the entry point for this executor. You must call spawn from within a future that is running through `block_on`.
	//
	pub fn block_on< F: Future >( &mut self, f: F ) -> F::Output
	{
		self.exec.borrow_mut().block_on( f )
	}
}



impl TryFrom<&mut Builder> for TokioCt
{
	type Error = std::io::Error;

	fn try_from( builder: &mut Builder ) -> Result<Self, Self::Error>
	{
		let exec = builder.basic_scheduler().build()?;

		Ok( Self
		{
			 handle  : exec.handle().clone()         ,
			 exec    : Rc::new( RefCell::new(exec) ) ,
		})
	}
}



impl Spawn for TokioCt
{
	fn spawn_obj( &self, future: FutureObj<'static, ()> ) -> Result<(), FutSpawnErr>
	{
		// We drop the JoinHandle, so the task becomes detached.
		//
		let _ = self.handle.spawn( future );

		Ok(())
	}
}



impl LocalSpawn for TokioCt
{
	fn spawn_local_obj( &self, future: LocalFutureObj<'static, ()> ) -> Result<(), FutSpawnErr>
	{
		// We transform the LocalFutureObj into a FutureObj, making it Send. Just magic!
		//
		// As long as the tokio basic scheduler is effectively keeping it's promise to run tasks on
		// the current thread, this should be fine. We made TokioCt !Send, to make sure it can't
		// be used from several threads and do not hand out tokio::runtime::Handle instances.
		//
		// As far as unwind safety goes, a warning has been added to TokioCt.
		//
		// This is necessary because tokio does not provide a handle that can spawn !Send futures.
		//
		let fut = unsafe { future.into_future_obj() };

		// We drop the JoinHandle, so the task becomes detached.
		//
		let _ = self.handle.spawn( fut );

		Ok(())
	}
}



#[ cfg(test) ]
//
mod tests
{
	use super::*;

	// It's important that this is not Send, as we allow spawning !Send futures on it.
	//
	static_assertions::assert_not_impl_any!( TokioCt: Send, Sync );
}
