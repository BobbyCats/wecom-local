# OpenCLI Integration

The first integration path should treat `wecom-local` as an external CLI.

```bash
opencli external register wecom-local \
  --binary wecom-local \
  --install "cargo install --git https://github.com/BobbyCats/wecom-local" \
  --desc "Query locally visible WeCom Desktop data"
```

An OpenCLI plugin can be added later if the project needs richer command
metadata or table-oriented output.
