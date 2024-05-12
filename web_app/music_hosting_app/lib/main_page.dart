import 'dart:convert';
// import 'dart:html';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;
import 'package:file_picker/file_picker.dart';
import 'package:dio/dio.dart';

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
  PlatformFile? _selectedFile;

  @override
  void initState() {
    super.initState();
    _checkTokenValidity();
  }

  Future<void> _checkTokenValidity() async {
    username = await getUsername() ?? "FAILED TO GET USERNAME";
    setState(() {
      _isTokenValid = false;
    });
    try {
      final token = (await getToken()) ?? "";
      if (token != "") {
        final response = await http.get(
            Uri.parse('http://localhost:3000/check_token'),
            headers: {'Authorization': token});
        if (response.statusCode == 200) {
          setState(() {
            _isTokenValid = true;
          });
        }
      }
    } catch (e) {}
  }

  Future<void> _logout() async {
    final token = (await getToken())!;
    storeToken(username, "");

    final response = await http.post(
      Uri.parse('http://localhost:3000/logout'),
      headers: {'authorization': token},
    );

    Navigator.pushReplacement(
        context,
        MaterialPageRoute(
          builder: (context) => MainPage(),
        ));
  }

  Future<void> _selectFile() async {
    FilePickerResult? result = await FilePicker.platform.pickFiles();
    if (result != null) {
      setState(() {
        _selectedFile = result.files.single;
        _selectFileStatus = 'Selected file: ${_selectedFile?.path}';
      });
    }
  }

  Future<void> _uploadTrack() async {
    String trackName = _trackNameController.text;
    if (trackName == "") {
      setState(() {
        _uploadTrackResult = "Name not specified";
      });
      return;
    }

    if (_selectedFile == null) {
      setState(() {
        _uploadTrackResult = "File not selected";
      });
      return;
    }

    String filename = _selectedFile?.name ?? "";
    String pathToFile = _selectedFile?.path ?? "";
    if (pathToFile == "") {
      return;
    }

    final formData = FormData.fromMap({
      "file": await MultipartFile.fromFile(pathToFile, filename: filename),
      "track_name": trackName
    });
    final token = (await getToken())!;
    final headers = {'authorization': token, "Content-Type": "multipart"};

    try {
      Response response = await Dio().post(
        'http://localhost:3000/upload_track',
        data: formData,
        options: Options(headers: headers),
      );

      if (response.statusCode == 201) {
        setState(() {
          _uploadTrackResult = 'Track uploaded';
          _selectFileStatus = 'File not selected';
          _trackNameController.clear();
        });
      } else {
        final statusCode = response.statusCode;
        // final body = response.body;
        // setState(() {_uploadTrackResult = 'Upload failed. response status code: $statusCode, body: $body';});
        setState(() {
          _uploadTrackResult =
              'Upload failed. response status code: $statusCode';
        });
      }
    } catch (e) {
      setState(() {
        _uploadTrackResult = 'Error occurred: $e. Please try again.';
      });
    }
  }

  @override
  Widget build(BuildContext context) {
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
                    : Colors.red,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
