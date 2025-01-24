debug_bin := "target/debug/grezzi_cli.exe"
release_bin := "target/release/grezzi_cli.exe"

release_dir := "release"

# Determine the file extension for binaries
# bin_ext := if env_var("OS") == "Windows_NT" { ".exe" } else { "" }

bin_ext := if os() == "windows" { ".exe" } else { "" }
alias b := build
alias re := release
alias c := clean
alias r := run

_default:
	@just -l

#build release
release:
	cargo b -r --all 
	mkdir -p {{release_dir}}
	cp target/release/grezzi_gui{{bin_ext}} {{release_dir}}/grezzi_gui{{bin_ext}}
	cp target/release/grezzi_cli{{bin_ext}} {{release_dir}}/grezzi_cli{{bin_ext}}

#run
run:
	cargo r 

#run the gui
gui:
	cargo r --package grezzi_gui

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

