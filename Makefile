.PHONY: init build

default: help

include .env.default

ifneq (,$(wildcard ./.env))
include .env
endif

help:
	@echo "$$HEADER"
	@sed \
		-e '/^[a-zA-Z0-9_\-]*:.*##/!d' \
		-e 's/:.*##\s*/:/' \
		-e 's/^\(.\+\):\(.*\)/$(shell tput setaf 6)\1$(shell tput sgr0):\2/' \
		$(MAKEFILE_LIST) | column -c2 -t -s :

init: ## Quick init from a Dilla ds in a repository, must include DS=... and REPO=git@...
ifndef REPO
	$(error [ERROR] URL is missing, ie: REPO=git@gitlab.com:dilla-io/ds/w3c_1.git)
endif
ifneq ($(findstring git@,$(REPO)),git@)
	$(error [ERROR] URL must be a valid git repository, given: $(REPO))
endif
ifndef DS
	$(error [ERROR] DS is missing, ie: DS=w3c_1)
endif
	@./scripts/ds.sh clone_url $(DS) -r $(REPO) -v
	@./scripts/pre_build.sh run $(DS) --skip-login --skip-check -v
	@./scripts/build-bg.sh docker_run $(DS) -v

check: ## Check and validate the templates in prebuild step, must include DS=... as DS=w3c_1
ifndef DS
	$(error [ERROR] DS is missing, ie: DS=w3c_1)
endif
	@./scripts/pre_build.sh run $(DS) -v

build: ## Build WASM Bindgen for local Dilla ds, must include DS=... as DS=w3c_1
ifndef DS
	$(error [ERROR] DS is missing, ie: DS=w3c_1)
endif
	@./scripts/pre_build.sh run $(DS) --skip-login --skip-check --skip-pull -v
	@./scripts/build-bg.sh docker_run $(DS) -v

build-component: ## Build WASM Component for local Dilla ds, must include DS=... as DS=w3c_1
ifndef DS
	$(error [ERROR] DS is missing, ie: DS=w3c_1)
endif
	@./scripts/pre_build.sh run $(DS) --skip-login --skip-check --skip-pull -v
	@./scripts/build-co.sh docker_run $(DS) -v

build-extism: ## Build WASM Extism for local Dilla ds, must include DS=... as DS=w3c_1
ifndef DS
	$(error [ERROR] DS is missing, ie: DS=w3c_1)
endif
	@./scripts/pre_build.sh run $(DS) --skip-login --skip-check --skip-pull -v
	@./scripts/build-ex.sh docker_run $(DS) -v
