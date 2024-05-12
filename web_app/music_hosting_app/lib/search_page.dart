import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:file_picker/file_picker.dart';
import 'package:dio/dio.dart';

import 'shared_state.dart';

class SearchPage extends StatefulWidget {
  @override
  _SearchPageState createState() => _SearchPageState();
}

class _SearchPageState extends State<SearchPage> {
  TextEditingController _searchRequestController = TextEditingController();
  String _searchResult = '';

  Future<void> _search() async {
    // final String username = _usernameController.text;
    // final String password = _passwordController.text;

    // final Map<String, String> body = {
    //   'username': username,
    //   'password': password,
    // };

    // try {
    //   final response = await http.post(
    //     Uri.parse('http://localhost:3000/signup'),
    //     headers: {'Content-Type': 'application/json'},
    //     body: jsonEncode(body),
    //   );

    //   if (response.statusCode == 201) {
    //     setState(() {
    //       _signupResult = 'You were registered';
    //     });
    //   } else {
    //     setState(() {
    //       _signupResult = 'Signup failed. Please try again.';
    //     });
    //   }
    // } catch (e) {
    //   setState(() {
    //     _signupResult = 'Error occurred. Please try again.';
    //   });
    // }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Search'),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            TextField(
              controller: _searchRequestController,
              decoration: InputDecoration(
                labelText: 'Search request',
              ),
            ),
            SizedBox(height: 16),
            ElevatedButton(
              onPressed: _search,
              child: Text('Search'),
            ),
            Text(
              _searchResult,
              style: TextStyle(
                color: _searchResult == '?????????????????'
                    ? Colors.green
                    : Colors.red,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
