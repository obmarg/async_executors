#![ cfg( feature = "localpool" ) ]

// Tested:
//
// ✔ pass a &mut LocalPool to a function that takes exec: `&mut impl SpawnExt`
// ✔ pass a &mut LocalPool to a function that takes exec: `&mut impl LocalSpawnExt`
// ✔ pass a      LocalPool to a function that takes exec: `impl SpawnExt      + Clone`
// ✔ pass a      LocalPool to a function that takes exec: `impl LocalSpawnExt + Clone`
//
mod common;

use
{
	common          :: * ,
	async_executors :: * ,
	futures         :: { channel::mpsc, executor::block_on, StreamExt },
};


// pass a &mut LocalPool to a function that takes exec: `&mut impl SpawnExt`
//
#[ test ]
//
fn test_spawn()
{
	let (tx, mut rx) = mpsc::channel( 1 );
	let mut exec = LocalPool::new();

	increment( 4, &mut exec, tx );
	exec.run();

	let result = block_on( rx.next() ).expect( "Some" );

		assert_eq!( 5u8, result );
}


// pass a &mut LocalPool to a function that takes exec: `&mut impl LocalSpawnExt`
//
#[ test ]
//
fn test_spawn_local()
{
	let (tx, mut rx) = mpsc::channel( 1 );
	let mut exec = LocalPool::new();

	increment_local( 4, &mut exec, tx );
	exec.run();

	let result = block_on( rx.next() ).expect( "Some" );

		assert_eq!( 5u8, result );
}


// pass a &mut LocalPool to a function that takes exec: `impl LocalSpawnExt + Clone`
//
#[ test ]
//
fn test_spawn_handle()
{
	let (tx, mut rx) = mpsc::channel( 1 );
	let mut exec = LocalPool::new();

	increment_by_value( 4, exec.handle(), tx );
	exec.run();

	let result = block_on( rx.next() ).expect( "Some" );

		assert_eq!( 5u8, result );
}


// pass a &mut LocalPool to a function that takes exec: `impl LocalSpawnExt + Clone`
//
#[ test ]
//
fn test_spawn_handle_local()
{
	let (tx, mut rx) = mpsc::channel( 1 );
	let mut exec = LocalPool::new();

	increment_by_value_local( 4, exec.handle(), tx );
	exec.run();

	let result = block_on( rx.next() ).expect( "Some" );

		assert_eq!( 5u8, result );
}
