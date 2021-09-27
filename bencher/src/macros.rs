/// Run benches in WASM environment.
///
/// Update Cargo.toml by adding:
/// ```toml
/// ..
/// [package]
/// name = "your-module"
/// ..
/// [[bench]]
/// name = 'module_benches'
/// harness = false
/// required-features = ['bench']
///
/// [features]
/// bench = [
///    'orml-bencher/bench'
///    'orml-weight-meter/bench'
/// ]
/// ..
/// ```
///
/// Create a file `benches/module_benches.rs` must be the same as bench name.
/// ```.ignore
/// use your_module::mock::{AllPalletsWithSystem, Block};
/// orml_bencher::run_benches!(AllPalletsWithSystem, Block);
/// ```
///
/// Define benches
///
/// Create a file `src/benches.rs`:
/// ```.ignore
/// #!#[cfg(feature = "bench")]
///
/// use orml_bencher::{Bencher, benches};
/// use crate::mock::*;
///
/// fn foo(b: &mut Bencher) {
///     b.prepare(|| {
///         // optional. prepare block, runs before bench
///     })
///     .bench(|| {
///         // foo must have macro `[orml_weight_meter::weight(..)]`
///         YourModule::foo();
///     })
///     .verify(|| {
///         // optional. verify block, runs before bench
///     });
/// }
///
/// fn bar(b: &mut Bencher) {
///     // optional. method name is used by default i.e: `bar`
///     b.name("bench_name")
///     .bench(|| {
///         // bar must have macro `[orml_weight_meter::weight(..)]`
///         YourModule::bar();
///     });
/// }
///
/// benches!(foo, bar); // Tests are generated automatically
/// ```
/// Update `src/lib.rs`:
/// ```.ignore
/// #[cfg(any(feature = "bench", test))]
/// pub mod mock; /* mock runtime needs to be compiled into wasm */
/// pub mod benches;
/// ```
///
/// Run benchmarking: `cargo bench --features=bench`
#[macro_export]
macro_rules! benches {
    ($($method:path),+) => {
        #[cfg(feature = "bench")]
        $crate::sp_core::wasm_export_functions! {
            fn run_benches() -> $crate::sp_std::vec::Vec<$crate::BenchResult> {
                let mut bencher = $crate::Bencher::default();
                $(
                    bencher.reset();
                    $method(&mut bencher);
                    if bencher.name.len() == 0 {
                        // use method name as default bench name
                        bencher.name(stringify!($method));
                    }
                    bencher.run();
                )+
                bencher.results
            }
        }

        #[cfg(all(feature = "bench", not(feature = "std")))]
        #[panic_handler]
        #[no_mangle]
        fn panic_handler(info: &::core::panic::PanicInfo) -> ! {
            unsafe {
                let message = $crate::sp_std::alloc::format!("{}", info);
                $crate::bench::print_error(message.as_bytes().to_vec());
                core::arch::wasm32::unreachable();
            }
        }

        // Tests
        mod tests {
            $(
                $crate::paste::item! {
                    #[test]
                    fn [<bench_ $method>] () {
                        $crate::sp_io::TestExternalities::new_empty().execute_with(|| {
                            let mut bencher = $crate::Bencher::default();
                            super::$method(&mut bencher);
                            bencher.run();
                        });
                    }
                }
            )+
        }

    }
}

#[macro_export]
macro_rules! run_benches {
	(
        $all_pallets_with_system:ident,
        $block:tt
    ) => {
		#[cfg(all(feature = "std", feature = "bench"))]
		#[main]
		pub fn main() -> std::io::Result<()> {
			use $crate::frame_benchmarking::frame_support::traits::StorageInfoTrait;
			let wasm = $crate::build_wasm::build()?;
			let storage_info = $all_pallets_with_system::storage_info();
			match $crate::bench_runner::run::<$block>(wasm) {
				Ok(output) => {
					$crate::handler::handle(output, storage_info);
				}
				Err(e) => {
					eprintln!("{:?}", e);
				}
			};
			Ok(())
		}
	};
}
