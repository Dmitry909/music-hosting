import 'package:flutter/foundation.dart';
import 'package:http/http.dart' as http;
import 'dart:convert';

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
  int _currentTrackId = -1;
  bool _newTrackAdded = false;

  void setCurrentTrackId(int currentTrackId) {
    _currentTrackId = currentTrackId;
    _newTrackAdded = true;
    notifyListeners();
  }

  void goToNextTrack() async {
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
      _currentTrackId = id;
    } else {
      throw Exception('Response of get_next_track is not 200');
    }
    _newTrackAdded = true;
    notifyListeners();
  }

  int getCurrentTrackId() {
    return _currentTrackId;
  }

  bool releaseNewTrackAdded() {
    final result = _newTrackAdded;
    _newTrackAdded = false;
    return result;
  }

  // void clearAndAddToQueue(int value) {
  //   _queue.clear();
  //   print('Cleared queue');
  //   addToQueue(value);
  // }

  // void addToQueue(int value) {
  //   _queue.add(value);
  //   print('Added $value to the queue');
  //   notifyListeners();
  // }

  // int removeFromQueue() {
  //   if (_queue.isNotEmpty) {
  //     final res = _queue.removeAt(0);
  //     notifyListeners();
  //     return res;
  //   }
  //   throw Exception("Tried to pop from empty queue");
  // }
}
