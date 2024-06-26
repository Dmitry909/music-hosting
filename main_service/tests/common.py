import requests
import json
import os.path
import filecmp

# host = 'http://localhost:3000'
host = 'http://158.160.49.125:3000'

def signup(username: str, password: str):
    json_data = {"username": username, "password": password}
    response = requests.post(f'{host}/signup', json=json_data)
    return response


def login(username: str, password: str):
    json_data = {"username": username, "password": password}
    response = requests.post(f'{host}/login', json=json_data)
    return response


def logout(token: str):
    response = requests.post(
        f'{host}/logout', headers={"Authorization": token})
    return response


def delete_account(username: str, password: str):
    json_data = {"username": username, "password": password}
    response = requests.delete(f'{host}/delete_account', json=json_data)
    return response


def check_token(token: str):
    response = requests.get(f'{host}/check_token',
                            headers={"Authorization": token})
    return response


def upload_track(token: str, track_name: str, file_path: str):
    with open(file_path, 'rb') as f:
        files = {
            'file': (file_path, f, 'audio/mpeg'),
            'track_name': (None, track_name, 'application/text')
        }

        response = requests.post(
            f'{host}/upload_track', headers={"Authorization": token}, files=files)

    assert (response.status_code == 201)
    obj = json.loads(response.text)
    assert ('id' in obj)
    assert (isinstance(obj['id'], int))
    assert (len(obj) == 1)
    id = obj['id']

    return id


def delete_track(token: str, track_id: int):
    json_data = {"track_id": track_id}
    response = requests.delete(
        f'{host}/delete_track', headers={"Authorization": token}, json=json_data)
    return response


def download_track(track_id: int):
    response = requests.get(f'{host}/download_track?id={track_id}')
    return response


def create_playlist(token: str, name: str):
    json_data = {'name': name}
    response = requests.post(
        f'{host}/create_playlist', headers={"Authorization": token}, json=json_data)

    assert (response.status_code == 201)
    obj = json.loads(response.text)
    assert ('id' in obj)
    assert (isinstance(obj['id'], int))
    assert (len(obj) == 1)
    playlist_id = obj['id']

    return playlist_id


def delete_playlist(token: str, playlist_id: int):
    json_data = {'playlist_id': playlist_id}
    response = requests.delete(
        f'{host}/delete_playlist', headers={"Authorization": token}, json=json_data)
    return response


def add_to_playlist(token: str, playlist_id: int, track_id: int):
    json_data = {'playlist_id': playlist_id, 'track_id': track_id}
    response = requests.put(f'{host}/add_to_playlist',
                            headers={"Authorization": token}, json=json_data)
    assert (response.status_code == 200)
    return response


def get_playlist(playlist_id: int):
    response = requests.get(f'{host}/get_playlist?playlist_id={playlist_id}')
    assert (response.status_code == 200)
    obj = json.loads(response.text)
    return obj


def search(query: str):
    response = requests.get(f'{host}/search?query={query}')

    assert (response.status_code == 200)
    obj = json.loads(response.content)
    assert (isinstance(obj, list))
    ids = list()
    for el in obj:
        assert (isinstance(el, dict))
        assert ('id' in el)
        assert (isinstance(el['id'], int))
        ids.append(el['id'])
    return ids


def get_next_track(token: str):
    response = requests.get(f'{host}/get_next_track', headers={"Authorization": token})

    assert(response.status_code == 200)
    obj = json.loads(response.content)
    assert (isinstance(obj, dict))
    assert ('id' in obj)
    assert (isinstance(obj['id'], int))

    return obj['id']
