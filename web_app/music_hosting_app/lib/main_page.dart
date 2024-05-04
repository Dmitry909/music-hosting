import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;
import 'package:file_picker/file_picker.dart';

import 'signup_page.dart';
import 'login_page.dart';
import 'shared_state.dart';

class MainPage extends StatefulWidget {
  @override
  _MainPageState createState() => _MainPageState();
}

class _MainPageState extends State<MainPage> {
  bool _isTokenValid = false;
  String username = "USERNAME INITIAL VALUE";
  TextEditingController _trackNameController = TextEditingController();
  String _selectFileStatus = 'File not selected';
  String _uploadTrackResult = '';
  late PlatformFile selectedFile;

  @override
  void initState() {
    print('Inside initState');
    super.initState();
    print('Calling _checkTokenValidity');
    _checkTokenValidity();
  }

  Future<void> _checkTokenValidity() async {
    print('Called _checkTokenValidity');
    username = await getUsername() ?? "FAILED TO GET USERNAME";
    setState(() {
      _isTokenValid = false;
    });
    print('Called _checkTokenValidity 2');
    try {
      print('try');
      final token = (await getToken()) ?? "";
      print('token: ');
      print(token);
      print('TOKEN FINISH');
      if (token != "") {
        final response = await http.get(
            Uri.parse('http://localhost:3002/check_token'),
            headers: {'Authorization': token});
        print(response);
        if (response.statusCode == 200) {
          setState(() {
            _isTokenValid = true;
          });
        }
      }
    } catch (e) {
      print("fgh");
    }

    print(_isTokenValid);
  }

  Future<void> _logout() async {
    final token = (await getToken())!;
    storeToken(username, "");

    final response = await http.post(
      Uri.parse('http://localhost:3002/logout'),
      headers: {'authorization': token},
    );

    print(response);

    Navigator.pushReplacement(
        context,
        MaterialPageRoute(
          builder: (context) => MainPage(),
        ));
  }

  Future<void> _uploadTrack() async {
    String trackName = _trackNameController.text;

    // TODO
    final Map<String, String> body = {
      'track_name': trackName,
    };

    try {
      final response = await http.post(
        Uri.parse('http://localhost:3002/upload_track'),
        headers: {'Content-Type': 'application/json'}, // TODO
        body: jsonEncode(body),
      );

      if (response.statusCode == 201) {
        setState(() {
          _uploadTrackResult = 'Track uploaded';
        });
      } else {
        final statusCode = response.statusCode;
        final body = response.body;
        setState(() {
          _uploadTrackResult =
              'Upload failed. response status code: $statusCode, body: $body';
        });
      }
    } catch (e) {
      setState(() {
        _uploadTrackResult = 'Error occurred. Please try again.';
      });
    }
  }

  Future<void> _selectFile() async {
    FilePickerResult? result = await FilePicker.platform.pickFiles();

    if (result != null) {
      selectedFile = result.files.first;
      setState(() {
        _selectFileStatus = 'Selected file: ${selectedFile.name}';
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    print('Inside build MainWidget');
    return _isTokenValid
        ? _buildWelcomePage(context)
        : _buildLoginSignupPage(context);
  }

  Widget _buildLoginSignupPage(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Main Page'),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            ElevatedButton(
              onPressed: () {
                Navigator.push(
                  context,
                  MaterialPageRoute(builder: (context) => LoginPage()),
                );
              },
              child: Text('Log in'),
            ),
            SizedBox(height: 16),
            ElevatedButton(
              onPressed: () {
                Navigator.push(
                  context,
                  MaterialPageRoute(builder: (context) => SignupPage()),
                );
              },
              child: Text('Sign up'),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildWelcomePage(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Hello, $username!'),
        actions: [
          Padding(
            padding: const EdgeInsets.only(right: 10.0),
            child: Align(
              alignment: Alignment.centerRight,
              child: ElevatedButton(
                onPressed: _logout,
                child: Text('Log out'),
              ),
            ),
          ),
        ],
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            TextField(
              controller: _trackNameController,
              decoration: InputDecoration(
                labelText: 'Track name',
              ),
            ),
            SizedBox(height: 16),
            ElevatedButton(
              onPressed: _selectFile,
              child: Text('Select File'),
            ),
            Text(
              _selectFileStatus,
              style: TextStyle(
                color: _selectFileStatus == 'File not selected'
                    ? Colors.red
                    : Colors.green,
              ),
            ),
            SizedBox(height: 16),
            ElevatedButton(
              onPressed: _uploadTrack,
              child: Text('Upload track'),
            ),
            Text(
              _uploadTrackResult,
              style: TextStyle(
                  color: _uploadTrackResult == 'Track uploaded'
                      ? Colors.green
                      : Colors.red,),
            ),
          ],
        ),
      ),
    );
  }
}
