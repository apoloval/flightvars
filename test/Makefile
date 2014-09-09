DEFINES=-DBOOST_LOG_DYN_LINK
INCLUDES=-I../include
CPP_FLAGS=-std=c++11
LD_FLAGS=-lboost_unit_test_framework -lboost_system -lboost_log-mt -lboost_log_setup-mt -lboost_thread-mt

TESTS=$(shell find . -name '*.hpp')
HEADERS=$(shell find ../include -name '*.hpp')

all: flightvars-test

test: flightvars-test
	./flightvars-test

flightvars-test: main.o
	${CXX} ${LD_FLAGS} -o flightvars-test main.o

main.o: main.cpp ${TESTS} ${HEADERS}
	${CXX} -c -o main.o ${DEFINES} ${CPP_FLAGS} ${INCLUDES} main.cpp

.PHONY clean:
	rm -f flightvars-test *.o