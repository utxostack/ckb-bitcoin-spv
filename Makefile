CARGO := @cargo

MOLC := moleculec
MOLC_VERSION := 0.7.5

NEXTEST_RUN_ARGS := --no-fail-fast --success-output never --failure-output final

#
# Generate Codes
#

.PHONY: generate-protocols
GEN_MOL_IN_DIR := verifier/schemas
GEN_MOL_OUT_DIR := verifier/src/types/generated
GEN_MOL_FILES := ${GEN_MOL_OUT_DIR}/types.rs
generate-protocols: check-moleculec-version ${GEN_MOL_FILES}

${GEN_MOL_OUT_DIR}/%.rs: ${GEN_MOL_IN_DIR}/%.mol
	${MOLC} --language rust --schema-file $< | rustfmt > $@

.PHONY: check-moleculec-version
check-moleculec-version:
	test "$$(${MOLC} --version | awk '{ print $$2  }' | tr -d ' ')" = ${MOLC_VERSION}

#
# Check
#

check:
	${CARGO} check --workspace

fmt:
	${CARGO} fmt --all --check

clippy:
	${CARGO} clippy --workspace --tests -- --deny warnings

test:
	${CARGO} nextest run ${NEXTEST_RUN_ARGS} --workspace

#
# Build
#

doc:
	${CARGO} doc --workspace --no-deps

build:
	${CARGO} build --workspace

release:
	${CARGO} build --workspace --release
