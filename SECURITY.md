# Security

Read the limits/security section in your [documentation language](README.md). NanoCode is an experimental visual transport, not an authentication system. CRC is not a signature. Codes can be copied and replayed.

Treat payloads as untrusted bytes. Authorize server-side before financial, physical access or privileged operations. Do not encode credentials. The core receives bounded packed images; native FFI callers must supply valid non-overlapping buffers. Browser rendering must not interpolate decoded payloads into HTML.

Report exploitable issues using GitHub private vulnerability reporting when enabled. If unavailable, contact the maintainer through their GitHub profile before posting sensitive reproduction details publicly. Never attach private camera frames or live credentials.

No production security support or remediation SLA is promised for the experimental 0.x series.
