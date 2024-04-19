import requests
import json
import os.path
import filecmp


def delete_account(username: str, username_ids: list):
    json_data = {"username": username}

    response = requests.delete(
        'http://localhost:3001/delete_account', json=json_data)

    assert (response.status_code == 200)
    for id in username_ids:
        assert (not os.path.exists(f'../tracks/{id}.mp3'))


def upload_track(username: str, track_name: str, file_path: str):
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
            'http://localhost:3001/upload_track', files=files)

    assert (response.status_code == 201)
    obj = json.loads(response.text)
    assert ('id' in obj)
    assert (isinstance(obj['id'], int))
    assert (len(obj) == 1)
    id = obj['id']
    assert (os.path.isfile(f'../tracks/{id}.mp3'))
    assert (filecmp.cmp(f'../tracks/{id}.mp3', file_path, shallow=False))

    return id


id_a = upload_track('alex', 'Porokh', 'test_tracks/a.mp3')
id_b = upload_track('alex', 'Mimino', 'test_tracks/b.mp3')
id_c = upload_track('alex', 'Pelevino', 'test_tracks/c.mp3')

delete_account('alex', [id_a, id_b, id_c])
