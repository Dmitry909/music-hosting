import requests
import json
import os.path
import filecmp

host = 'http://localhost:3002'


def delete_account(username: str, username_ids: list):
    json_data = {"username": username}

    response = requests.delete(
        f'{host}/delete_account', json=json_data)

    assert (response.status_code == 200)
    for id in username_ids:
        assert (not os.path.exists(f'../tracks/{id}.mp3'))


def upload_track_raw(username: str, track_name: str, file_path: str):
    json_data = {
        "username": username,
        "track_name": track_name
    }

    with open(file_path, 'rb') as f:
        files = {
            'file': (file_path, f, 'audio/mpeg'),
            'json': (None, json.dumps(json_data), 'application/json')
        }

        response = requests.post(
            f'{host}/upload_track', files=files)
    return response


def upload_track(username: str, track_name: str, file_path: str):
    response = upload_track_raw(username, track_name, file_path)
    assert (response.status_code == 201)
    obj = json.loads(response.text)
    assert ('id' in obj)
    assert (isinstance(obj['id'], int))
    assert (len(obj) == 1)
    id = obj['id']
    assert (os.path.isfile(f'../tracks/{id}.mp3'))
    assert (filecmp.cmp(f'../tracks/{id}.mp3', file_path, shallow=False))

    return id


def delete_track(username: str, track_id: int):
    json_data = {
        "username": username,
        "track_id": track_id
    }

    response = requests.delete(
        f'{host}/delete_track', json=json_data)

    assert (response.status_code == 200)
    assert (not os.path.exists(f'../tracks/{track_id}.mp3'))


def download_track(track_id: int):
    response = requests.get(
        f'{host}/download_track?id={track_id}')

    assert (response.status_code == 200)
    return response.content


def search(query: str, expected_ids: set):
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
    assert (ids == expected_ids)
    return obj


def change_rate(track_id: int, cnt_rates_delta: int, sum_rates_delta: int):
    json_data = {
        "track_id": track_id,
        "cnt_rates_delta": cnt_rates_delta,
        "sum_rates_delta": sum_rates_delta,
    }

    response = requests.put(
        f'{host}/change_rate', json=json_data)

    assert (response.status_code == 200)
