debug_bin := "target/debug/grezzi_calculator.exe"
release_bin := "target/release/grezzi_calculator.exe"

alias b := build
alias re := release
alias c := clean
alias r := run

_default:
	@just -l

#build release
release:
	cargo b -r 
	mkdir -p release
	cp {{release_bin}} ./release

#run
run:
	cargo r

#build
build:
	cargo b

#clean
clean:
	cargo clean 
	rm -fr test

#create test directory
test: build
	cp {{debug_bin}} ./test/

