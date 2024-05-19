import 'package:flutter/foundation.dart';
import 'package:http/http.dart' as http;
import 'dart:convert';
import 'dart:collection';

import 'package:shared_preferences/shared_preferences.dart';

Future<void> storeToken(String username, String token) async {
  SharedPreferences prefs = await SharedPreferences.getInstance();
  await prefs.setString('username', username);
  await prefs.setString('authToken', token);
}

Future<String?> getUsername() async {
  SharedPreferences prefs = await SharedPreferences.getInstance();
  return prefs.getString('username');
}

Future<String?> getToken() async {
  SharedPreferences prefs = await SharedPreferences.getInstance();
  return prefs.getString('authToken');
}

class PlayerData with ChangeNotifier {
  int _currentPos = -1;
  List<int> _history = [];
  bool _newTrackAdded = false;

  void setNewTrackId(int currentTrackId) {
    _history.clear();
    _history.add(currentTrackId);
    _currentPos = 0;
    _newTrackAdded = true;
    notifyListeners();
  }

  void goToPreviousTrack() {
    print('goToPreviousTrack called');
    print(_currentPos);
    if (_currentPos >= 1) {
      _currentPos -= 1;
      _newTrackAdded = true;
      notifyListeners();
    }
  }

  void goToNextTrack() async {
    print('goToNextTrack called');
    if (_currentPos >= 0 && _currentPos + 1 < _history.length) {
      _currentPos += 1;
      _newTrackAdded = true;
      notifyListeners();
      return;
    }

    final token = (await getToken())!;

    final response = await http.get(
      Uri.parse('http://localhost:3000/get_next_track'),
      headers: {'authorization': token},
    );

    print(response.statusCode);

    if (response.statusCode == 200) {
      print(response.body);
      Map<String, dynamic> data = jsonDecode(response.body);
      if (data['id'] == null) {
        throw Exception('No id in response of get_next_track');
      }
      int id = data['id'];

      _history.add(id);
      _currentPos += 1;
      _newTrackAdded = true;
      notifyListeners();
    } else {
      throw Exception('Response of get_next_track is not 200');
    }
  }

  int getCurrentTrackId() {
    if (_currentPos < 0 || _currentPos >= _history.length) {
      return -1;
    }
    return _history[_currentPos];
  }

  bool releaseNewTrackAdded() {
    final result = _newTrackAdded;
    _newTrackAdded = false;
    return result;
  }
}
