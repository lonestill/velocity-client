import base64
svg = '<svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 40 40"><text x="20" y="28" font-size="24" font-weight="bold" fill="#00fff5" text-anchor="middle" font-family="system-ui">V</text></svg>'
b64 = base64.b64encode(svg.encode()).decode()
print('data:image/svg+xml;base64,' + b64)
