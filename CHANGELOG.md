# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog], and this project adheres to
[Semantic Versioning]. The file is auto-generated using [Conventional Commits].

[keep a changelog]: https://keepachangelog.com/en/1.0.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html
[conventional commits]: https://www.conventionalcommits.org/en/v1.0.0/

## Overview
- [`0.2.0`](#020) – _2021.07.05_
- [`0.1.1`](#011) – _2021.07.05_
- [`0.1.0`](#010) – _2021.05.20_
## [0.2.0]

_2021.07.05_

### Contributions

This release is made possible by the following people (in alphabetical order).
Thank you all for your contributions. Your work – no matter how significant – is
greatly appreciated by the community. 💖

- Jan Christian Grünhage (<jan.christian@gruenhage.xyz>)

### Changes

#### Features

- **add yggdrasil v0.4 support** ([`76a1a3a`])

  This commit stops generating encryption keys and just generates signing
  keys, because yggdrasil v0.4 doesn't use encryption keys anymore.
  Address generation has also been updated.


## [0.1.1]

_2021.07.05_

### Contributions

This release is made possible by the following people (in alphabetical order).
Thank you all for your contributions. Your work – no matter how significant – is
greatly appreciated by the community. 💖

- Jan Christian Grünhage (<jan.christian@gruenhage.xyz>)

### Changes

#### Bug Fixes

- **switch secret/public keys** ([`8cdc536`])

  The order of the keys was switched around when generating the output
  file, meaning we ended up with the secret keys in the `_pub`, and the
  public keys in `_sec`. Ooops.


## [0.1.0]

_2021.05.20_

Initial release


### Changes



<!--
Config(
  accept_types: ["feat", "fix", "perf"],
  type_headers: {
    "feat": "Features",
    "fix": "Bug Fixes",
    "perf": "Performance Improvements"
  }
)

Template(
# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog], and this project adheres to
[Semantic Versioning]. The file is auto-generated using [Conventional Commits].

[keep a changelog]: https://keepachangelog.com/en/1.0.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html
[conventional commits]: https://www.conventionalcommits.org/en/v1.0.0/

## Overview

{%- for release in releases %}
- [`{{ release.version }}`](#{{ release.version | replace(from=".", to="") }}) – _{{ release.date | date(format="%Y.%m.%d")}}_
{%- endfor %}

{%- for release in releases %}
## [{{ release.version }}]

_{{ release.date | date(format="%Y.%m.%d") }}_
{%- if release.notes %}

{{ release.notes }}
{% endif -%}
{%- if release.changeset.contributors %}

### Contributions

This release is made possible by the following people (in alphabetical order).
Thank you all for your contributions. Your work – no matter how significant – is
greatly appreciated by the community. 💖
{% for contributor in release.changeset.contributors %}
- {{ contributor.name }} (<{{ contributor.email }}>)
{%- endfor %}
{%- endif %}

### Changes

{% for type, changes in release.changeset.changes | group_by(attribute="type") -%}

#### {{ type | typeheader }}

{% for change in changes -%}
- **{{ change.description }}** ([`{{ change.commit.short_id }}`])

{% if change.body -%}
{{ change.body | indent(n=2) }}

{% endif -%}
{%- endfor -%}

{% endfor %}
{%- endfor -%}
)
-->
