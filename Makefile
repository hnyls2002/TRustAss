SRC = tra/src/

backtrace:
	cd $(SRC) && RUST_BACKTRACE=1 cargo run

.PHONY: backtrace