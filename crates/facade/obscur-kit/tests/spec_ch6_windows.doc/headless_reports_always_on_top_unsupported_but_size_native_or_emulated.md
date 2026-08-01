§6.2 Dynamic window properties: "Operations return typed results." with
"Capability result categories:" / "- native;" / "- emulated;" /
"- conditional;" / "- unsupported."

Headless has no compositor, so always-on-top is `unsupported` and the
property never applies; a plain size change is `native` or `emulated` and
does apply. Two calls, one result type, different categories.
