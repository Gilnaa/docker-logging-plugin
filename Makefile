# Plugin parameters
PLUGIN_NAME=gilnaa/file-logging-plugin
PLUGIN_TAG=0.0.1
PLUGIN_BUILD_DIR=build/plugin

.PHONY: all protogen clean plugin-docker-image rootfs build-plugin enable push

.NOTPARALLEL:
all: clean plugin-docker-image rootfs build-plugin

src/generated_protos/entry.rs: protos/entry.proto
	mkdir -p src/generated_protos
	protoc --rust_out src/generated_protos protos/entry.proto

protogen: src/generated_protos/entry.rs

clean:
	rm -rf ${PLUGIN_BUILD_DIR}

plugin-docker-image:
	@echo "Building Docker image for plugin rootfs"
	docker build -t ${PLUGIN_NAME}:rootfs .

rootfs:
	@echo "Create rootfs in ${PLUGIN_BUILD_DIR}/rootfs"
	mkdir -p ${PLUGIN_BUILD_DIR}/rootfs
	docker create --name tmprootfs ${PLUGIN_NAME}:rootfs
	docker export tmprootfs | tar -x -C ${PLUGIN_BUILD_DIR}/rootfs
	docker rm -vf tmprootfs
	cp config.json ${PLUGIN_BUILD_DIR}/

build-plugin:
	@echo "Removing existing plugin ${PLUGIN_NAME}:${PLUGIN_TAG} if it exists"
	docker plugin rm -f ${PLUGIN_NAME}:${PLUGIN_TAG} || true
	@echo "Building plugin ${PLUGIN_NAME}:${PLUGIN_TAG} in ${PLUGIN_BUILD_DIR}"
	docker plugin create ${PLUGIN_NAME}:${PLUGIN_TAG} ${PLUGIN_BUILD_DIR}

enable:
	docker plugin enable ${PLUGIN_NAME}:${PLUGIN_TAG}

push: clean docker rootfs create enable
	docker plugin push ${PLUGIN_NAME}:${PLUGIN_TAG}
