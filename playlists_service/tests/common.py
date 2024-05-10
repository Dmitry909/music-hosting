import requests
import json

host = 'http://localhost:3003'

def get_field(text, field):
    obj = json.loads(text)
    assert (field in obj)
    assert (isinstance(obj[field], int))
    return obj[field]


def create_playlist(username: str, playlist_name: str):
    json_data = {
        'username': username, 
        'name': playlist_name
    }
    response = requests.post(f'{host}/create_playlist', json=json_data)
    return response


def add_to_playlist(username: str, playlist_id: int, track_id: int):
    json_data = {
        'username': username, 
        'playlist_id': playlist_id, 
        'track_id': track_id
    }
    response = requests.put(f'{host}/add_to_playlist', json=json_data)
    return response


def get_playlist(playlist_id: int):
    json_data = {'playlist_id': playlist_id}
    response = requests.get(f'{host}/get_playlist', json=json_data)
    return response


def delete_from_playlist(username: str, playlist_id: int, track_id: int):
    json_data = {'username': username, 'playlist_id': playlist_id, 'track_id': track_id}
    response = requests.delete(f'{host}/delete_from_playlist', json=json_data)
    return response


def delete_playlist(username: str, playlist_id: int):
    json_data = {'username': username, 'playlist_id': playlist_id}
    response = requests.delete(f'{host}/delete_playlist', json=json_data)
    return response


def delete_account(username: str):
    json_data = {'username': username}
    response = requests.delete(f'{host}/delete_account', json=json_data)
    return response
