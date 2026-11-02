CC=g++
CDEFINES=
SOURCES=Dispatcher.cpp Mode.cpp precomp.cpp profanity.cpp SpeedSample.cpp
OBJECTS=$(SOURCES:.cpp=.o)
EXECUTABLE=profanity.x64

UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
	LDFLAGS=-framework OpenCL
	CFLAGS=-c -std=c++11 -Wall -mmmx -O2
else
	LDFLAGS=-s -lOpenCL -mcmodel=large
	CFLAGS=-c -std=c++11 -Wall -mmmx -O2 -mcmodel=large
endif

all: $(SOURCES) $(EXECUTABLE)

.PHONY: vuke brainflayer

# Rust generator (vuke/) - needs a Rust toolchain
vuke:
	cd vuke && cargo build --release

# brainflayer-MultiBlooms/ - needs gcc, OpenSSL, GMP, autotools
brainflayer:
	$(MAKE) -C brainflayer-MultiBlooms

$(EXECUTABLE): $(OBJECTS)
	$(CC) $(OBJECTS) $(LDFLAGS) -o $@

.cpp.o:
	$(CC) $(CFLAGS) $(CDEFINES) $< -o $@

clean:
	rm -rf *.o *.x64

distclean: clean
	$(MAKE) -C brainflayer-MultiBlooms distclean || true
	rm -rf vuke/target
