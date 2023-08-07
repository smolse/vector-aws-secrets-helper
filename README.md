# vector-aws-secrets-helper

A helper tool for [Vector](https://vector.dev/) to securely retrieve secrets from AWS SSM Parameter Store and AWS
Secrets Manager using the [exec](https://vector.dev/highlights/2022-07-07-secrets-management/) backend.

## Installation

Download an executable for the target platform from the
[releases page](https://github.com/smolse/vector-aws-secrets-helper/releases) or clone the repo and build it with the
`cargo build` command. Place the executable in a directory that is in your (or, actually, in the Vector user's) `PATH`
environment variable, e.g. `/usr/local/bin`.

## Usage

Once the executable is installed, it can be used as described in the
[Vector documentation](https://vector.dev/docs/reference/configuration/global-options/#secret.exec). The tool uses
the [default credential provider chain](https://docs.aws.amazon.com/sdkref/latest/guide/standardized-credentials.html#credentialProviderChain)
to authenticate to AWS.

Here is an example configuration for the `exec` secrets backend in Vector:

```toml
[secret.aws_ssm]
type = "exec"
command = ["/usr/local/bin/vector-aws-secrets-helper", "ssm"]

[secret.aws_secrets_manager]
type = "exec"
command = ["/usr/local/bin/vector-aws-secrets-helper", "secretsmanager"]
```

## Limitations

While it's idiomatic to use `/` in the names of SSM Parameter Store parameters and Secrets Manager secrets to create a
hierarchy, Vector currently does not support slashes in the secret names. The only supported characters are
alphanumeric, underscores and dots. Here are some examples of valid secret references (for both SSM Parameter Store and
Secrets Manager):
- `SECRET[aws_ssm.secret]`
- `SECRET[aws_ssm.another_one]`
- `SECRET[aws_ssm.one.more]`
- `SECRET[aws_ssm..secret.with.a.leading.comma]`
