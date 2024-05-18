import 'package:shared_preferences/shared_preferences.dart';
import 'package:flutter/foundation.dart';

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

class QueueModel with ChangeNotifier {
  final List<int> _queue = [];

  List<int> get queue => _queue;

  void clearAndAddToQueue(int value) {
    _queue.clear();
    print('Cleared queue');
    addToQueue(value);
  }

  void addToQueue(int value) {
    _queue.add(value);
    print('Added $value to the queue');
    notifyListeners();
  }

  int removeFromQueue() {
    if (_queue.isNotEmpty) {
      final res = _queue.removeAt(0);
      notifyListeners();
      return res;
    }
    throw Exception("Tried to pop from empty queue");
  }
}
