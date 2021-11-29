# SPDX-License-Identifier: GPL-3.0-or-later
# This file is part of Neatcoin.
#
# Copyright (c) 2019-2021 Wei Tang.
# Copyright (c) 2019 Polkasource.
#
# Neatcoin is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# Neatcoin is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with Neatcoin. If not, see <http://www.gnu.org/licenses/>.

# ===== START FIRST STAGE ======
FROM phusion/baseimage:0.11 as builder
LABEL maintainer "wei@that.world"
LABEL description="Neatcoin builder."

ARG PROFILE=release
ARG STABLE=1.56.1
WORKDIR /rustbuilder
COPY . /rustbuilder/neatcoin

# PREPARE OPERATING SYSTEM & BUILDING ENVIRONMENT
RUN apt-get update && \
	apt-get install -y cmake pkg-config libssl-dev git clang libclang-dev

# UPDATE RUST DEPENDENCIES
ENV RUSTUP_HOME "/rustbuilder/.rustup"
ENV CARGO_HOME "/rustbuilder/.cargo"
RUN curl -sSf https://sh.rustup.rs | sh -s -- --default-toolchain none -y
ENV PATH "$PATH:/rustbuilder/.cargo/bin"
RUN rustup update $STABLE

# BUILD RUNTIME AND BINARY
RUN rustup target add wasm32-unknown-unknown --toolchain $STABLE
RUN cd /rustbuilder/neatcoin && RUSTUP_TOOLCHAIN=$STABLE WASM_BUILD_TOOLCHAIN=$STABLE RUSTC_BOOTSTRAP=1 RANDOMX_ARCH=default cargo build --$PROFILE --locked
# ===== END FIRST STAGE ======

# ===== START SECOND STAGE ======
FROM phusion/baseimage:0.11
LABEL maintainer "wei@that.world"
LABEL description="Neatcoin binary."
ARG PROFILE=release
COPY --from=builder /rustbuilder/neatcoin/target/$PROFILE/neatcoin /usr/local/bin

# REMOVE & CLEANUP
RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	rm -rf /usr/lib/python* && \
	mkdir -p /root/.local/share/neatcoin && \
	ln -s /root/.local/share/neatcoin /data
RUN	rm -rf /usr/bin /usr/sbin

# FINAL PREPARATIONS
EXPOSE 30333 9933 9944
VOLUME ["/data"]
#CMD ["/usr/local/bin/neatcoin"]
WORKDIR /usr/local/bin
ENTRYPOINT ["neatcoin"]
CMD ["--chain=neatcoin"]
# ===== END SECOND STAGE ======
