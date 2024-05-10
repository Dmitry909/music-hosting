from common import *
import random
from string import ascii_uppercase, digits


def random_str(length):
    return ''.join(random.choice(ascii_uppercase + digits) for _ in range(length))


def test_create_add_get_delete():
    username = random_str(10)
    playlist_name1 = random_str(10)
    create_response = create_playlist(username, playlist_name1)
    assert(create_response.status_code == 201)
    playlist_id1 = get_field(create_response.text, 'id')

    add_response1 = add_to_playlist(username, playlist_id1, 123)
    assert(add_response1.status_code == 200)
    add_response2 = add_to_playlist(username, playlist_id1, 456)
    assert(add_response2.status_code == 200)
    
    get_response = get_playlist(playlist_id1)
    assert(get_response.status_code == 200)
    playlist = json.loads(get_response.text)
    assert(playlist == [123, 456])

    delete_from_playlist_response = delete_from_playlist(username, playlist_id1, 123)
    assert(delete_from_playlist_response.status_code == 200)
    get_response = get_playlist(playlist_id1)
    playlist = json.loads(get_response.text)
    assert(playlist == [456])

    playlist_name2 = random_str(10)
    create_response = create_playlist(username, playlist_name2)
    playlist_id2 = get_field(create_response.text, 'id')
    get_response = get_playlist(playlist_id2)
    assert(get_response.status_code == 200)
    playlist = json.loads(get_response.text)
    assert(playlist == [])

    delete_playlist_response = delete_playlist(username, playlist_id2)
    assert(delete_playlist_response.status_code == 200)
    get_response = get_playlist(playlist_id2)
    assert(get_response.status_code == 404)

    playlist_name3 = random_str(10)
    create_response = create_playlist(username, playlist_name3)
    playlist_id3 = get_field(create_response.text, 'id')

    delete_account_response = delete_account(username)
    assert(delete_account_response.status_code == 200)
    assert(get_playlist(playlist_id1).status_code == 404)
    assert(get_playlist(playlist_id2).status_code == 404)
    assert(get_playlist(playlist_id3).status_code == 404)

    print('test_create_add_get_delete OK')


test_create_add_get_delete()
