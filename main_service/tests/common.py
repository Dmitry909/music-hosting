import requests
import json
import os.path
import filecmp

host = 'http://localhost:3002'

def signup(username: str, password: str):
    json_data = {"username": username, "password": password}
    response = requests.post(f'{host}/signup', json=json_data)
    return response


def login(username: str, password: str):
    json_data = {"username": username, "password": password}
    response = requests.post(f'{host}/login', json=json_data)
    return response


def logout(token: str):
    response = requests.post(f'{host}/logout', headers={"Authorization": token})
    return response


def delete_account(username: str, password: str):
    json_data = {"username": username, "password": password}
    response = requests.delete(f'{host}/delete_account', json=json_data)
    return response


def upload_track(token: str, username: str, track_name: str, file_path: str):
    json_data = {"username": username, "track_name": track_name}

    with open(file_path, 'rb') as f:
        files = {
            'file': (file_path, f, 'audio/mpeg'),
            'json': (None, json.dumps(json_data), 'application/json')
        }

        response = requests.post(f'{host}/upload_track', headers={"Authorization": token}, files=files)

    assert (response.status_code == 201)
    obj = json.loads(response.text)
    assert ('id' in obj)
    assert (isinstance(obj['id'], int))
    assert (len(obj) == 1)
    id = obj['id']

    return id


def delete_track(token: str, username: str, track_id: int):
    json_data = {"username": username, "track_id": track_id}
    response = requests.delete(f'{host}/delete_track', headers={"Authorization": token}, json=json_data)
    return response


def download_track(track_id: int):
    response = requests.get(f'{host}/download_track?id={track_id}')
    return response


def search(query: str):
    response = requests.get(f'{host}/search?query={query}')

    assert (response.status_code == 200)
    obj = json.loads(response.content)
    assert (isinstance(obj, list))
    ids = set()
    for el in obj:
        assert (isinstance(el, dict))
        assert ('id' in el)
        assert (isinstance(el['id'], int))
        ids.add(el['id'])
    return ids
