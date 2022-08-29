# livebook-indicator

A GTK3 based Linux "desktop app" for [Livebook](https://livebook.dev), on the same lines as the Livebook ["desktop apps" for macOS and Windows](https://news.livebook.dev/introducing-the-livebook-desktop-app-4C8dpu) (= an app indicator).

This app is meant to go with the fully contained Livebook build currently developed over at https://github.com/livebook-dev/livebook/compare/main...mz2:livebook:mz2/snap (it is [built inside](https://github.com/mz2/livebook/blob/mz2/snap/snap/snapcraft.yaml#L30-L33) that package).

## Installing & running

The indicator is packaged in the [experimental Livebook snap package](https://github.com/mz2/livebook/tree/mz2/snap), and is available on the Snap Store (presently unlisted and only on the edge channel). To install:

```bash
sudo snap install livebook --edge
```

Once installed, you can start the indicator with:

```bash
livebook.indicator
```

## Building & running locally

To build this gtk-rs dependent app with `cargo`, a number of prerequisite packages are needed. As an example in Ubuntu 20.04:

```
sudo apt install \
pkg-config \
libglib2.0-dev \
libatk1.0-dev \
libcairo2-dev \
libpango1.0-dev \
libgdk-pixbuf2.0-dev \
libgtk-3-dev \
libglib2.0-0 \
libuuid1 \
zlib1g \
libpcre3 \
libgdk-pixbuf2.0-0 \
gir1.2-atk-1.0 \
gir1.2-glib-2.0 \
libcairo2 \
libpango-1.0-0
```

You can then build & run livebook-indicator locally using Cargo.

```bash
cargo run
```

Perhaps the easier way to meet above local build and runtime prerequisites is to build the indicator is as part of the livebook Snap package locally: https://github.com/mz2/livebook/tree/mz2/snap

```bash
git clone https://github.com/mz2/livebook --branch mz2/snap
snapcraft # to install snapcraft on your distro of choice: `sudo snap install snapcraft --classic`
```

## Known issues

The indicator is under development and you're probably best off not using it quite yet. You should be aware of at least the following:

- No desktop launcher when packaged inside the snap.

## License

Copyright (C) 2022 Matias Piipari.

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
