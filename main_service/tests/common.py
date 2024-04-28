import requests
import json
import os.path
import filecmp

host = 'http://localhost:3002'

def signup(username: str, password: str):
    json_data = {"username": username, "password": password}
    response = requests.post(f'{host}/signup', json=json_data)
    assert(response.status_code == 201)


def login(username: str, password: str):
    json_data = {"username": username, "password": password}
    response = requests.post(f'{host}/login', json=json_data)
    assert(response.status_code == 200)
    return response.headers["Authorization"]


def logout(token: str):
    response = requests.post(f'{host}/logout', headers={"Authorization": token})
    assert(response.status_code == 200)
