#!/usr/bin/fish
if not test -e ./benchmarks/dacapo-2006-10-MR2.jar
	mkdir -p benchmarks 
	wget https://downloads.sourceforge.net/project/dacapobench/archive/2006-10-MR2/dacapo-2006-10-MR2.jar -O benchmarks/dacapo-2006-10-MR2.jar
end

if test -d ./jikesrvm/rust_mmtk
	rmdir ./jikesrvm/rust_mmtk
	ln -s ../           ./jikesrvm/rust_mmtk
	ln -s ../benchmarks ./jikesrvm/benchmarks
end

cd jikesrvm
eval $argv