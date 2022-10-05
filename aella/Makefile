%.o: %.s
	as -arch arm64 -o $@ $^

%.exe: %.o
	ld -o $@ $^ -lSystem -syslibroot `xcrun -sdk macosx --show-sdk-path` -e _start -arch arm64

clean:
	rm *.o *.exe
