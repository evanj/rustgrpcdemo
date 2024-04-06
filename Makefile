BUILD_DIR:=target
PROTOC:=$(BUILD_DIR)/bin/protoc

all: $(PROTOC) $(BUILD_DIR)/.proto_format_stamp
	PATH=$$PATH:$(pwd)/target/bin cargo build
	PATH=$$PATH:$(pwd)/target/bin cargo test
	PATH=$$PATH:$(pwd)/target/bin cargo clippy

# Ensures proto files are formatted: must be added as a dependency before the generated files
$(BUILD_DIR)/.proto_format_stamp: $(wildcard proto/*.proto) | $(BUILD_DIR)
	clang-format --style=Google -i $<
	touch $@

# download protoc to a temporary tools directory
$(PROTOC): $(BUILD_DIR)/getprotoc | $(BUILD_DIR)
	$(BUILD_DIR)/getprotoc --outputDir=$(BUILD_DIR)

$(BUILD_DIR)/getprotoc: | $(BUILD_DIR)
	GOBIN=$(realpath $(BUILD_DIR)) go install github.com/evanj/hacks/getprotoc@latest

$(BUILD_DIR):
	mkdir -p $@

clean:
	$(RM) -r $(PROTOC) $(BUILD_DIR)/.proto_format_stamp
