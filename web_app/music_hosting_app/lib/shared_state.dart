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

class TrackInfo {
  int id = 0;
  String name = "";
  String authorUsername = "";
  int cntRates = 0;
  int sumRates = 0;

  TrackInfo(
      this.id, this.name, this.authorUsername, this.cntRates, this.sumRates);
}

class PlayerData with ChangeNotifier {
  int _currentPos = -1;
  List<TrackInfo> _history = [];
  bool _newTrackAdded = false;

  void setNewTrack(TrackInfo newTrackInfo) {
    _history.clear();
    _history.add(newTrackInfo);
    _currentPos = 0;
    _newTrackAdded = true;
    notifyListeners();
  }

  void goToPreviousTrack() {
    if (_currentPos >= 1) {
      _currentPos -= 1;
      _newTrackAdded = true;
      notifyListeners();
    }
  }

  void goToNextTrack() async {
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

    if (response.statusCode == 200) {
      Map<String, dynamic> data = jsonDecode(utf8.decode(response.bodyBytes));
      if (data['id'] == null) {
        throw Exception('No id in response of get_next_track');
      }
      int id = data['id'];
      String name = data['name'];
      String authorUsername = data['author_username'];
      int cntRates = data['cnt_rates'];
      int sumRates = data['sum_rates'];
      final trackInfo = TrackInfo(id, name, authorUsername, cntRates, sumRates);

      _history.add(trackInfo);
      _currentPos += 1;
      _newTrackAdded = true;
      notifyListeners();
    } else {
      throw Exception('Response of get_next_track is not 200');
    }
  }

  TrackInfo getCurrentTrackInfo() {
    if (_currentPos < 0 || _currentPos >= _history.length) {
      return TrackInfo(-1, "", "", 0, 0);
    }
    return _history[_currentPos];
  }

  bool releaseNewTrackAdded() {
    final result = _newTrackAdded;
    _newTrackAdded = false;
    return result;
  }
}
