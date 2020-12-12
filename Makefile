VERSION != grep '^version' Cargo.toml | grep -Po '\d+\.\d+\.\d+'

.PHONY: tag

help:
	@echo "Make targets:"
	@echo "  tag     Create Git tag using version from Cargo.toml"
	@echo "  help    Print this help"

tag:
	git tag -am "Version ${VERSION}" v${VERSION}
